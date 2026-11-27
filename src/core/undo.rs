//! Undo support for Spark.
//!
//! When a template is extracted to disk, Spark records an [`UndoLog`]: one
//! [`UndoEntry`] per filesystem target, capturing its path and the bytes that
//! were there **before** the write (`None` when the file did not exist yet).
//! [`UndoLog::apply`] reverses the extraction — newly created files are deleted
//! and overwritten/patched files are restored to their previous content.
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
}

/// Outcome of [`UndoLog::apply`]: how many files were restored, deleted, or
/// skipped, plus any non-fatal errors encountered along the way.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UndoReport {
    /// Files whose previous content was written back.
    pub restored: usize,
    /// Newly created files that were removed.
    pub deleted: usize,
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

    /// `true` when there is nothing to undo.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
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
    /// patched files are restored to their previous content. Processing
    /// continues past individual failures, which are collected in
    /// [`UndoReport::errors`]. Entries are processed in reverse order so files
    /// created last are removed first.
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
        };
        let report = log.apply();
        assert_eq!(report.restored, 1);
        assert_eq!(fs::read_to_string(&nested).unwrap(), "back");
    }
}
