//! Undo support for Spark.
//!
//! When a template is extracted to disk, Spark records an [`UndoLog`]: one
//! [`UndoEntry`] per filesystem target, capturing its path and the bytes that
//! were there **before** the write (`None` when the file did not exist yet).
//! [`UndoLog::apply`] reverses the extraction — newly created files are deleted
//! and overwritten/patched files are restored to their previous content.
//!
//! The log also records the directories Spark had to create to hold those
//! files; undo removes them (deepest first, and only while empty) so reversing
//! an extraction does not leave empty folders behind.
//!
//! Non-file targets (`stdout://`, `stderr://`, `clipboard://`) are not
//! recorded: there is nothing on disk to reverse.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// A single reversible filesystem write performed during extraction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UndoEntry {
    /// Absolute path that was written.
    pub path: PathBuf,
    /// Content present at `path` before the write, or `None` if the file was
    /// newly created (so undo deletes it).
    pub previous: Option<String>,
}

impl UndoEntry {
    /// Records a write to `path`. `previous` is the prior on-disk content
    /// (`None` for a file that did not exist before).
    pub fn new(path: impl Into<PathBuf>, previous: Option<String>) -> Self {
        Self {
            path: path.into(),
            previous,
        }
    }

    /// `true` when this entry describes a file Spark created from scratch.
    pub fn is_new(&self) -> bool {
        self.previous.is_none()
    }
}

/// An ordered record of the filesystem writes from a single extraction.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UndoLog {
    /// Writes in the order they were committed.
    pub entries: Vec<UndoEntry>,
    /// Directories Spark created to hold the written files. Undo removes these
    /// deepest-first, and only while empty, so it never deletes a directory the
    /// user already had or one that still holds other files. Defaulted so logs
    /// written by older versions (which lacked this field) still load.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub created_dirs: Vec<PathBuf>,
}

/// Outcome of [`UndoLog::apply`]: how many files were restored, deleted, or
/// skipped, plus any non-fatal errors encountered along the way.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UndoReport {
    /// Files whose previous content was written back.
    pub restored: usize,
    /// Newly created files that were removed.
    pub deleted: usize,
    /// Newly created directories that were removed (empty) after their files.
    pub removed_dirs: usize,
    /// Entries that needed no action (e.g. a created file already gone).
    pub skipped: usize,
    /// Human-readable, non-fatal errors (one per entry that failed).
    pub errors: Vec<String>,
}

impl UndoLog {
    /// Creates an empty log.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends an entry.
    pub fn push(&mut self, entry: UndoEntry) {
        self.entries.push(entry);
    }

    /// Records a directory Spark created so undo can remove it (if still empty).
    pub fn push_dir(&mut self, dir: impl Into<PathBuf>) {
        self.created_dirs.push(dir.into());
    }

    /// `true` when there is nothing to undo.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.created_dirs.is_empty()
    }

    /// Serializes the log to `path` as pretty JSON, creating parent
    /// directories as needed.
    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let json = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        fs::write(path, json)
    }

    /// Loads a log previously written by [`UndoLog::save`].
    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let json = fs::read_to_string(path)?;
        serde_json::from_str(&json).map_err(io::Error::other)
    }

    /// Reverses the recorded writes. Created files are deleted; overwritten or
    /// patched files are restored to their previous content. Once the files are
    /// handled, any directories Spark created are removed deepest-first, but
    /// only while empty, so a directory the user already had — or one that still
    /// holds other files — is left in place. Processing continues past
    /// individual failures, which are collected in [`UndoReport::errors`].
    /// Entries are processed in reverse order so files created last are removed
    /// first.
    pub fn apply(&self) -> UndoReport {
        let mut report = UndoReport::default();

        for entry in self.entries.iter().rev() {
            match &entry.previous {
                Some(content) => {
                    if let Some(parent) = entry.path.parent() {
                        if !parent.as_os_str().is_empty() {
                            let _ = fs::create_dir_all(parent);
                        }
                    }
                    match fs::write(&entry.path, content) {
                        Ok(()) => report.restored += 1,
                        Err(e) => report
                            .errors
                            .push(format!("could not restore {}: {e}", entry.path.display())),
                    }
                }
                None => {
                    if !entry.path.exists() {
                        report.skipped += 1;
                        continue;
                    }
                    match fs::remove_file(&entry.path) {
                        Ok(()) => report.deleted += 1,
                        Err(e) => report
                            .errors
                            .push(format!("could not remove {}: {e}", entry.path.display())),
                    }
                }
            }
        }

        // Remove the directories Spark created, deepest-first so a child is
        // gone before its parent is considered. A directory is removed only
        // while empty (via `remove_dir`, never `remove_dir_all`), so one the
        // user already had, or one that still holds other files, is left alone.
        let mut dirs: Vec<&PathBuf> = self.created_dirs.iter().collect();
        dirs.sort_by_key(|d| std::cmp::Reverse(d.components().count()));
        for dir in dirs {
            if !dir.exists() {
                continue;
            }
            match fs::read_dir(dir) {
                // Still holds something (e.g. a file the user added); leave it.
                Ok(mut entries) => {
                    if entries.next().is_some() {
                        continue;
                    }
                }
                Err(e) => {
                    report
                        .errors
                        .push(format!("could not inspect {}: {e}", dir.display()));
                    continue;
                }
            }
            match fs::remove_dir(dir) {
                Ok(()) => report.removed_dirs += 1,
                Err(e) => report
                    .errors
                    .push(format!("could not remove directory {}: {e}", dir.display())),
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_deletes_created_files_and_restores_overwritten() {
        let dir_tmp = tempfile::tempdir().unwrap();
        let dir = dir_tmp.path();

        let created = dir.join("new.txt");
        let overwritten = dir.join("existing.txt");
        fs::write(&created, "spark made this").unwrap();
        fs::write(&overwritten, "NEW body").unwrap();

        let log = UndoLog {
            entries: vec![
                UndoEntry::new(created.clone(), None),
                UndoEntry::new(overwritten.clone(), Some("ORIGINAL body".into())),
            ],
            created_dirs: Vec::new(),
        };

        let report = log.apply();
        assert_eq!(report.deleted, 1);
        assert_eq!(report.restored, 1);
        assert!(report.errors.is_empty());
        assert!(!created.exists(), "created file should be gone");
        assert_eq!(
            fs::read_to_string(&overwritten).unwrap(),
            "ORIGINAL body",
            "overwritten file should be restored"
        );
    }

    #[test]
    fn apply_skips_created_file_that_is_already_gone() {
        let dir_tmp = tempfile::tempdir().unwrap();
        let missing = dir_tmp.path().join("ghost.txt");

        let log = UndoLog {
            entries: vec![UndoEntry::new(missing, None)],
            created_dirs: Vec::new(),
        };
        let report = log.apply();
        assert_eq!(report.skipped, 1);
        assert_eq!(report.deleted, 0);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir_tmp = tempfile::tempdir().unwrap();
        let manifest = dir_tmp.path().join("nested").join("undo.json");

        let log = UndoLog {
            entries: vec![
                UndoEntry::new(PathBuf::from("/a/b.txt"), None),
                UndoEntry::new(PathBuf::from("/a/c.txt"), Some("prev".into())),
            ],
            created_dirs: vec![PathBuf::from("/a")],
        };
        log.save(&manifest).unwrap();
        let loaded = UndoLog::load(&manifest).unwrap();
        assert_eq!(loaded, log);
    }

    #[test]
    fn apply_restores_even_when_parent_dir_was_removed() {
        let dir_tmp = tempfile::tempdir().unwrap();
        let nested = dir_tmp.path().join("gone").join("file.txt");

        // Parent does not exist yet; restore must recreate it.
        let log = UndoLog {
            entries: vec![UndoEntry::new(nested.clone(), Some("back".into()))],
            created_dirs: Vec::new(),
        };
        let report = log.apply();
        assert_eq!(report.restored, 1);
        assert_eq!(fs::read_to_string(&nested).unwrap(), "back");
    }

    #[test]
    fn apply_removes_created_dirs_deepest_first() {
        let dir_tmp = tempfile::tempdir().unwrap();
        let root = dir_tmp.path().join("proj");
        let nested = root.join("src");
        let file = nested.join("main.rs");
        fs::create_dir_all(&nested).unwrap();
        fs::write(&file, "fn main() {}").unwrap();

        let log = UndoLog {
            entries: vec![UndoEntry::new(file.clone(), None)],
            // Intentionally parent-before-child to prove apply sorts them.
            created_dirs: vec![root.clone(), nested.clone()],
        };
        let report = log.apply();
        assert_eq!(report.deleted, 1);
        assert_eq!(report.removed_dirs, 2);
        assert!(report.errors.is_empty());
        assert!(!nested.exists(), "nested dir should be gone");
        assert!(!root.exists(), "root dir should be gone");
    }

    #[test]
    fn apply_keeps_created_dir_that_is_not_empty() {
        let dir_tmp = tempfile::tempdir().unwrap();
        let root = dir_tmp.path().join("proj");
        let created = root.join("made.txt");
        let user_file = root.join("kept.txt");
        fs::create_dir_all(&root).unwrap();
        fs::write(&created, "spark made this").unwrap();
        fs::write(&user_file, "user added this").unwrap();

        let log = UndoLog {
            entries: vec![UndoEntry::new(created.clone(), None)],
            created_dirs: vec![root.clone()],
        };
        let report = log.apply();
        assert_eq!(report.deleted, 1);
        assert_eq!(report.removed_dirs, 0, "dir still holds a user file");
        assert!(report.errors.is_empty());
        assert!(root.exists(), "non-empty dir must be preserved");
        assert!(user_file.exists(), "user's file must survive");
    }

    #[test]
    fn apply_skips_created_dir_that_is_already_gone() {
        let dir_tmp = tempfile::tempdir().unwrap();
        let missing = dir_tmp.path().join("never-made");

        let log = UndoLog {
            entries: vec![],
            created_dirs: vec![missing],
        };
        let report = log.apply();
        assert_eq!(report.removed_dirs, 0);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn created_dirs_round_trip_through_save_load() {
        let dir_tmp = tempfile::tempdir().unwrap();
        let manifest = dir_tmp.path().join("undo.json");
        let log = UndoLog {
            entries: vec![UndoEntry::new(PathBuf::from("/a/b/c.txt"), None)],
            created_dirs: vec![PathBuf::from("/a/b"), PathBuf::from("/a")],
        };
        log.save(&manifest).unwrap();
        assert_eq!(UndoLog::load(&manifest).unwrap(), log);
    }

    #[test]
    fn old_manifest_without_created_dirs_still_loads() {
        let dir_tmp = tempfile::tempdir().unwrap();
        let manifest = dir_tmp.path().join("undo.json");
        // A manifest written before created_dirs existed has no such key.
        fs::write(
            &manifest,
            r#"{"entries":[{"path":"/a/b.txt","previous":null}]}"#,
        )
        .unwrap();
        let loaded = UndoLog::load(&manifest).unwrap();
        assert_eq!(loaded.entries.len(), 1);
        assert!(loaded.created_dirs.is_empty());
    }
}
