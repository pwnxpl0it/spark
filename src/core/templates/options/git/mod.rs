use colored::*;
use std::path::Path;
use std::process::Command;

pub fn check_git() -> Result<(), String> {
    Command::new("git")
        .arg("--version")
        .output()
        .map_err(|_| "Git is not installed. Please install git and try again.".to_string())?;
    Ok(())
}

/// Initializes a git repository inside `project_root` without changing the
/// current working directory of the calling process.
pub fn init(project_root: &str) {
    if let Err(e) = check_git() {
        eprintln!("{}: {}", "error".red().bold(), e.red().bold());
        return;
    }

    let root = Path::new(project_root);
    if !root.is_dir() {
        eprintln!(
            "{}: {}",
            "error".red().bold(),
            format!("Project root '{}' is not a directory", project_root)
                .red()
                .bold()
        );
        return;
    }

    if root.join(".git").exists() {
        println!("{}", "\n✅ Git is already initialized.".yellow().bold());
        return;
    }

    match Command::new("git").arg("init").current_dir(root).status() {
        Ok(status) if status.success() => {
            println!("{}", "\n✅ Git initialized successfully.".green().bold());
        }
        _ => {
            eprintln!(
                "{}: {}",
                "error".red().bold(),
                "Git initialization failed.".red().bold()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{check_git, init};

    #[test]
    fn check_git_succeeds_when_git_is_available() {
        assert!(check_git().is_ok());
    }

    #[test]
    fn init_does_not_change_process_cwd() {
        let dir = std::env::temp_dir().join("spark_test_git_init_cwd");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let before = std::env::current_dir().unwrap();
        init(&dir.to_string_lossy());
        let after = std::env::current_dir().unwrap();

        assert_eq!(before, after, "git::init must not chdir");
        assert!(
            dir.join(".git").is_dir(),
            "repository should be initialized"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn init_with_missing_root_does_not_panic() {
        init("/definitely/not/a/real/dir/spark");
    }
}
