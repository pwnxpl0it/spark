use colored::*;
use std::process::Command;

pub fn check_git() -> Result<(), String> {
    if Command::new("git").arg("--version").spawn().is_err() {
        return Err("Git is not installed. Please install git and try again.".to_string());
    }
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

    //if std::path::Path::new(".git").exists() {
        //println!("{}", "\n✅ Git is already initialized.".yellow().bold());
        //return;
    //}

    let cmd = Command::new("git")
        .arg("init") //TODO: maybe add git arguments? that can be a bit risky..
        .stderr(std::process::Stdio::null()) // hide hints and errors
        .status();

    if let Ok(status) = cmd {
        if status.success() {
            println!("{}", "\n✅ Git initialized successfully.".green().bold());
        } else {
            eprintln!(
                "{}: {}",
                "error".red().bold(),
                "Git initialization failed.".red().bold()
            );
        }
    } else {
        eprintln!(
            "{}: {}",
            "error".red().bold(),
            "Failed to run git command.".red().bold()
        );
    }
}
