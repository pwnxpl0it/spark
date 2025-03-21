use colored::*;
use std::process::Command;

pub fn check_git() -> Result<(), String> {
    Command::new("git")
        .arg("--version")
        .output()
        .map_err(|_| "Git is not installed. Please install git and try again.".to_string())?;
    Ok(())
}

pub fn init(project_root: &str) {
    if let Err(e) = check_git() {
        eprintln!("{}: {}", "error".red().bold(), e.red().bold());
        return;
    }

    if let Err(e) = std::env::set_current_dir(project_root) {
        eprintln!(
            "{}: {}",
            "error".red().bold(),
            format!("Failed to change directory: {}", e).red().bold()
        );
        return;
    }

    if std::path::Path::new(".git").exists() {
        println!("{}", "\n✅ Git is already initialized.".yellow().bold());
        return;
    }

    match Command::new("git").arg("init").status() {
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
