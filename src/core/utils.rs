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

#[cfg(test)]
mod tests {
    use super::{create_dirs, list_files, write_content};
    use std::fs;

    #[test]
    fn create_dirs_creates_nested_directories() {
        let dir_path = std::env::temp_dir().join("spark_test_create_dirs/nested");
        let _ = fs::remove_dir_all(dir_path.parent().unwrap());

        create_dirs(&dir_path.to_string_lossy());
        assert!(dir_path.is_dir());

        let _ = fs::remove_dir_all(dir_path.parent().unwrap());
    }

    #[test]
    fn write_content_replaces_init_pjname_placeholder() {
        let file_path = std::env::temp_dir().join("spark_test_write_content.txt");
        let _ = fs::remove_file(&file_path);

        write_content(&file_path, "Hello initPJNAME!")
            .expect("write_content should succeed");

        let read = fs::read_to_string(&file_path).expect("should read written file");
        assert_eq!(read, "Hello {{$PROJECTNAME}}!");

        let _ = fs::remove_file(&file_path);
    }

    #[test]
    fn list_files_returns_all_nested_files() {
        let test_dir = std::env::temp_dir().join("spark_test_list_files");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(test_dir.join("sub")).unwrap();

        let file1 = test_dir.join("file1.txt");
        let file2 = test_dir.join("file2.txt");
        let file3 = test_dir.join("sub").join("file3.txt");
        fs::write(&file1, "a").unwrap();
        fs::write(&file2, "b").unwrap();
        fs::write(&file3, "c").unwrap();

        let files = list_files(&test_dir).expect("list_files should succeed");
        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|f| f == &file1.to_string_lossy()));
        assert!(files.iter().any(|f| f == &file2.to_string_lossy()));
        assert!(files.iter().any(|f| f == &file3.to_string_lossy()));

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn list_files_errors_when_path_is_not_directory() {
        let file_path = std::env::temp_dir().join("spark_test_list_files_not_dir.txt");
        fs::write(&file_path, "x").unwrap();

        let err = list_files(&file_path).expect_err("should fail for non-directory");
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);

        let _ = fs::remove_file(&file_path);
    }
}
