//! Pre/post generation hooks.
//!
//! A template may declare shell commands to run before and after generation in
//! a top-level `[hooks]` table:
//!
//! ```toml
//! [hooks]
//! pre = ["command -v cargo"]
//! post = ["cargo fmt", "git init"]
//! ```
//!
//! Hooks run arbitrary commands, so Spark never executes them on its own: the
//! library exposes them as data and runs them only when the caller explicitly
//! asks (the CLI does so behind the `--run-hooks` flag). Commands run through
//! the platform shell (`sh -c` on Unix, `cmd /C` on Windows) in the current
//! working directory, in listed order, and the phase stops at the first command
//! that exits non-zero.

use crate::Error;
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Commands to run before and after a template is generated.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Hooks {
    /// Commands run before any files are written.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pre: Vec<String>,
    /// Commands run after all files are written (and git init, if enabled).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub post: Vec<String>,
}

impl Hooks {
    /// Returns `true` when no hook commands are defined.
    pub fn is_empty(&self) -> bool {
        self.pre.is_empty() && self.post.is_empty()
    }

    /// Returns the total number of hook commands across both phases.
    pub fn len(&self) -> usize {
        self.pre.len() + self.post.len()
    }

    /// Runs the `pre` commands in order, stopping at the first failure.
    pub fn run_pre(&self) -> crate::Result<()> {
        run_all("pre", &self.pre)
    }

    /// Runs the `post` commands in order, stopping at the first failure.
    pub fn run_post(&self) -> crate::Result<()> {
        run_all("post", &self.post)
    }
}

/// Runs every command in a phase, echoing each to stderr before executing it.
///
/// Returns `Err` as soon as a command fails to start or exits non-zero, so a
/// failing `pre` phase can abort generation before anything is written.
fn run_all(phase: &str, commands: &[String]) -> crate::Result<()> {
    for command in commands {
        eprintln!("hook ({phase}): {command}");
        let status = shell(command).status().map_err(|e| {
            Error::Custom(format!("hook ({phase}) '{command}' failed to start: {e}"))
        })?;
        if !status.success() {
            return Err(Error::Custom(match status.code() {
                Some(code) => format!("hook ({phase}) '{command}' exited with status {code}"),
                None => format!("hook ({phase}) '{command}' terminated by signal"),
            }));
        }
    }
    Ok(())
}

/// Builds the shell command used to run a single hook string.
#[cfg(not(windows))]
fn shell(command: &str) -> Command {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(command);
    cmd
}

/// Builds the shell command used to run a single hook string.
#[cfg(windows)]
fn shell(command: &str) -> Command {
    let mut cmd = Command::new("cmd");
    cmd.arg("/C").arg(command);
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_and_len_count_both_phases() {
        let hooks = Hooks::default();
        assert!(hooks.is_empty());
        assert_eq!(hooks.len(), 0);

        let hooks = Hooks {
            pre: vec!["a".into()],
            post: vec!["b".into(), "c".into()],
        };
        assert!(!hooks.is_empty());
        assert_eq!(hooks.len(), 3);
    }

    #[test]
    fn deserializes_from_toml() {
        let hooks: Hooks = toml::from_str(
            r#"
pre = ["command -v cargo"]
post = ["cargo fmt", "git init"]
"#,
        )
        .unwrap();
        assert_eq!(hooks.pre, vec!["command -v cargo".to_string()]);
        assert_eq!(
            hooks.post,
            vec!["cargo fmt".to_string(), "git init".to_string()]
        );
    }

    #[test]
    fn missing_phase_defaults_to_empty() {
        let hooks: Hooks = toml::from_str(r#"post = ["echo done"]"#).unwrap();
        assert!(hooks.pre.is_empty());
        assert_eq!(hooks.post, vec!["echo done".to_string()]);
    }

    #[cfg(not(windows))]
    #[test]
    fn run_pre_executes_commands_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join("marker");
        let hooks = Hooks {
            pre: vec![
                format!("printf 'a' >> {}", marker.display()),
                format!("printf 'b' >> {}", marker.display()),
            ],
            post: vec![],
        };
        hooks.run_pre().unwrap();
        assert_eq!(std::fs::read_to_string(&marker).unwrap(), "ab");
    }

    #[cfg(not(windows))]
    #[test]
    fn run_post_stops_at_first_failure() {
        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join("marker");
        let hooks = Hooks {
            pre: vec![],
            post: vec![
                "exit 3".into(),
                format!("printf 'should-not-run' >> {}", marker.display()),
            ],
        };
        let err = hooks.run_post().unwrap_err().to_string();
        assert!(err.contains("status 3"), "unexpected error: {err}");
        assert!(!marker.exists(), "later command ran after a failure");
    }
}
