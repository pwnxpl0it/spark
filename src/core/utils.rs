use colored::*;
use std::{fs, io, path::Path};
use walkdir::WalkDir;

pub fn create_dirs(dir: &str) {
    match shellexpand::full(dir) {
        Ok(expanded) => {
            if let Err(e) = fs::create_dir_all(expanded.as_ref()) {
                eprintln!("{}: {}", "error".red(), e);
            } else {
                println!(
                    "{}: {}",
                    "creating directory".blue(),
                    expanded.to_string().bold().green()
                );
            }
        }
        Err(e) => eprintln!("{}: Failed to expand path '{}': {}", "error".red(), dir, e),
    }
}

pub fn write_content<P: AsRef<Path>>(path: P, content: &str) -> std::io::Result<()> {
    let path_str = path.as_ref().to_string_lossy().to_string();

    let expanded_path = match shellexpand::full(&path_str) {
        Ok(expanded) => expanded.to_string(),
        Err(_) => path_str.clone(), // If expansion fails, use the original path
    };

    fs::write(
        Path::new(&expanded_path),
        content.replace("initPJNAME", "{{$PROJECTNAME}}"),
    )
    .map(|_| {
        println!(
            "{}: {}",
            "file written".blue(),
            expanded_path.bold().green()
        );
    })
}

pub fn list_files(dir: &Path) -> Result<Vec<String>, io::Error> {
    let mut files = Vec::new();

    if !dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Provided path is not a directory",
        ));
    }

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
    {
        files.push(entry.path().to_string_lossy().to_string());
    }

    Ok(files)
}
