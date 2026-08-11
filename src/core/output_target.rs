//! Output target abstraction for Spark.
//!
//! A [`File`](crate::File) path can now contain an explicit protocol scheme that
//! redirects rendered output to a sink other than the filesystem:
//!
//! | Path value      | Behaviour                               |
//! |-----------------|----------------------------------------|
//! | `stdout://`     | Write rendered content to **stdout**   |
//! | `stderr://`     | Write rendered content to **stderr**   |
//! | `file://some/path` | Write to the filesystem path after `file://` |
//! | anything else   | Write to the filesystem (unchanged behaviour) |
//!
//! ## Windows path safety
//!
//! `C:\path` and `C:/path` contain a single-character "scheme" (`C`).  The
//! parser only recognises schemes that are longer than one character, so
//! Windows drive letters are never mis-parsed as protocol targets.
//!
//! ## Adding new protocols
//!
//! Implement a new variant on [`OutputTarget`] and add the matching arm in
//! [`OutputTarget::from_path`] and [`OutputTarget::write`].  No changes to
//! the rendering pipeline are required.

use colored::Colorize;
use std::{
    io::{self, Write},
    path::PathBuf,
};

/// The sink to which rendered template output is directed.
#[derive(Debug, PartialEq)]
pub enum OutputTarget {
    /// Write to a filesystem path (shell-expanded).
    File(PathBuf),
    /// Write to standard output.
    Stdout,
    /// Write to standard error.
    Stderr,
}

impl OutputTarget {
    /// Parse a template path value into an [`OutputTarget`].
    ///
    /// Recognised schemes:
    /// - `stdout://` → [`OutputTarget::Stdout`]
    /// - `stderr://` → [`OutputTarget::Stderr`]
    /// - `file://<path>` → [`OutputTarget::File`] for `<path>`
    ///
    /// Everything else (including Windows drive-letter paths such as `C:\foo`)
    /// is treated as a plain filesystem path.
    pub fn from_path(path: &str) -> Self {
        // Extract the scheme: the part before the first ':'.
        // A scheme must be more than one character to avoid treating Windows
        // drive letters (e.g. `C:`) as protocol identifiers.
        if let Some(colon_pos) = path.find(':') {
            let scheme = &path[..colon_pos];
            if scheme.len() > 1 {
                match scheme {
                    "stdout" => return Self::Stdout,
                    "stderr" => return Self::Stderr,
                    "file" => {
                        // Strip "file://" prefix; the rest is the filesystem path.
                        let rest = path
                            .strip_prefix("file://")
                            .unwrap_or_else(|| path.strip_prefix("file:").unwrap_or(path));
                        return Self::File(PathBuf::from(rest));
                    }
                    _ => {
                        // Unknown scheme — fall through to plain file path so we
                        // don't break unusual-but-valid paths.
                    }
                }
            }
        }

        Self::File(PathBuf::from(path))
    }

    /// Write `content` to the target.
    ///
    /// For [`OutputTarget::File`] the path is shell-expanded (same behaviour
    /// as the existing [`crate::utils::write_content`]).
    pub fn write(&self, content: &str) -> std::io::Result<()> {
        match self {
            Self::Stdout => {
                print!("{}", content);
                io::stdout().flush()
            }
            Self::Stderr => {
                eprint!("{}", content);
                io::stderr().flush()
            }
            Self::File(path) => {
                let path_str = path.to_string_lossy();
                let expanded = match shellexpand::full(&path_str) {
                    Ok(e) => e.to_string(),
                    Err(_) => path_str.to_string(),
                };
                // Create parent directories if needed (mirrors the existing
                // behaviour that was previously in `prepare_file_content`).
                if let Some(parent) = std::path::Path::new(&expanded).parent() {
                    let parent_str = parent.to_string_lossy();
                    if !parent_str.is_empty() {
                        if let Err(e) = std::fs::create_dir_all(parent) {
                            eprintln!("{}: {}", "error".red(), e);
                        }
                    }
                }
                // Preserve the existing behaviour: replace the legacy
                // `initPJNAME` sentinel with `{{$PROJECTNAME}}`.
                std::fs::write(
                    std::path::Path::new(&expanded),
                    content.replace("initPJNAME", "{{$PROJECTNAME}}"),
                )
                .map(|_| {
                    println!("{}: {}", "file written".blue(), expanded.bold().green());
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OutputTarget;
    use std::path::PathBuf;

    // ── from_path parsing ────────────────────────────────────────────────────

    #[test]
    fn stdout_scheme_yields_stdout_target() {
        assert_eq!(OutputTarget::from_path("stdout://"), OutputTarget::Stdout);
    }

    #[test]
    fn stderr_scheme_yields_stderr_target() {
        assert_eq!(OutputTarget::from_path("stderr://"), OutputTarget::Stderr);
    }

    #[test]
    fn file_scheme_strips_prefix() {
        assert_eq!(
            OutputTarget::from_path("file://some/path/file.txt"),
            OutputTarget::File(PathBuf::from("some/path/file.txt"))
        );
    }

    #[test]
    fn file_scheme_without_double_slash() {
        // "file:" without "//" should still strip the "file:" prefix.
        assert_eq!(
            OutputTarget::from_path("file:relative/path.txt"),
            OutputTarget::File(PathBuf::from("relative/path.txt"))
        );
    }

    #[test]
    fn relative_path_yields_file_target() {
        assert_eq!(
            OutputTarget::from_path("src/main.rs"),
            OutputTarget::File(PathBuf::from("src/main.rs"))
        );
    }

    #[test]
    fn absolute_unix_path_yields_file_target() {
        assert_eq!(
            OutputTarget::from_path("/usr/local/bin/spark"),
            OutputTarget::File(PathBuf::from("/usr/local/bin/spark"))
        );
    }

    #[test]
    fn windows_drive_letter_is_not_a_scheme() {
        // "C" is a single character before ':', so it must NOT be treated as a
        // protocol scheme.
        assert_eq!(
            OutputTarget::from_path(r"C:\Users\foo\project"),
            OutputTarget::File(PathBuf::from(r"C:\Users\foo\project"))
        );
        assert_eq!(
            OutputTarget::from_path("C:/Users/foo/project"),
            OutputTarget::File(PathBuf::from("C:/Users/foo/project"))
        );
    }

    #[test]
    fn unknown_multi_char_scheme_falls_back_to_file() {
        // An unrecognised scheme keeps backward-compat by treating the whole
        // string as a filesystem path.
        assert_eq!(
            OutputTarget::from_path("ftp://some/path"),
            OutputTarget::File(PathBuf::from("ftp://some/path"))
        );
    }

    #[test]
    fn plain_filename_no_colon_is_file() {
        assert_eq!(
            OutputTarget::from_path("README.md"),
            OutputTarget::File(PathBuf::from("README.md"))
        );
    }

    // ── write behaviour ──────────────────────────────────────────────────────

    #[test]
    fn stdout_write_succeeds() {
        // We cannot easily capture stdout in a unit test without additional
        // dependencies; we simply verify the call does not error.
        let result = OutputTarget::Stdout.write("hello from stdout\n");
        assert!(result.is_ok());
    }

    #[test]
    fn stderr_write_succeeds() {
        let result = OutputTarget::Stderr.write("hello from stderr\n");
        assert!(result.is_ok());
    }

    #[test]
    fn file_write_creates_file_with_content() {
        let dir = std::env::temp_dir().join("spark_test_output_target_file");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let file_path = dir.join("out.txt");
        let target = OutputTarget::File(file_path.clone());
        target.write("hello spark").unwrap();

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "hello spark");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_write_replaces_init_pjname_placeholder() {
        let dir = std::env::temp_dir().join("spark_test_output_target_pjname");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let file_path = dir.join("tpl.toml");
        let target = OutputTarget::File(file_path.clone());
        target.write("path = initPJNAME").unwrap();

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "path = {{$PROJECTNAME}}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── from_path + write round-trips ────────────────────────────────────────

    #[test]
    fn from_path_stdout_then_write_succeeds() {
        let target = OutputTarget::from_path("stdout://");
        assert_eq!(target, OutputTarget::Stdout);
        assert!(target.write("roundtrip stdout\n").is_ok());
    }

    #[test]
    fn from_path_stderr_then_write_succeeds() {
        let target = OutputTarget::from_path("stderr://");
        assert_eq!(target, OutputTarget::Stderr);
        assert!(target.write("roundtrip stderr\n").is_ok());
    }

    #[test]
    fn from_path_file_scheme_writes_to_disk() {
        let dir = std::env::temp_dir().join("spark_test_output_target_file_scheme");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let file_path = dir.join("result.txt");
        let raw = format!("file://{}", file_path.display());
        let target = OutputTarget::from_path(&raw);

        target.write("via file://").unwrap();
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "via file://");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
