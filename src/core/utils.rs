use crate::output_target::OutputTarget;
use std::io;
use std::path::Path;

/// Writes `content` to `path` on the filesystem, creating parent directories.
/// Thin wrapper over [`OutputTarget::File`].
pub fn write_content<P: AsRef<Path>>(path: P, content: &str) -> std::io::Result<()> {
    OutputTarget::File(path.as_ref().to_path_buf()).write(content)
}

/// Lists every regular file under `dir`, honouring `.gitignore` rules and
/// skipping the `.git` directory itself. Hidden files such as `.gitignore`
/// and `.github/` are included.
pub fn list_files(dir: &Path) -> Result<Vec<String>, io::Error> {
    if !dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Provided path is not a directory",
        ));
    }

    let walker = ignore::WalkBuilder::new(dir)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .require_git(false)
        .filter_entry(|entry| entry.file_name() != ".git")
        .build();

    let mut files = Vec::new();
    for entry in walker.filter_map(Result::ok) {
        if entry.file_type().is_some_and(|t| t.is_file()) {
            files.push(entry.path().to_string_lossy().to_string());
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::{list_files, write_content};
    use std::fs;

    #[test]
    fn write_content_writes_verbatim() {
        let file_path_tmp = tempfile::tempdir().unwrap();
        let file_path = file_path_tmp.path().join("spark_test_write_content.txt");

        write_content(&file_path, "Hello initPJNAME!").expect("write_content should succeed");

        let read = fs::read_to_string(&file_path).expect("should read written file");
        assert_eq!(read, "Hello initPJNAME!");
    }

    #[test]
    fn list_files_returns_all_nested_files() {
        let test_dir_tmp = tempfile::tempdir().unwrap();
        let test_dir = test_dir_tmp.path().to_path_buf();
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
    }

    #[test]
    fn list_files_includes_dotfiles_and_skips_git_dir_and_ignored() {
        let test_dir_tmp = tempfile::tempdir().unwrap();
        let test_dir = test_dir_tmp.path().to_path_buf();
        fs::create_dir_all(test_dir.join(".git/objects")).unwrap();
        fs::create_dir_all(test_dir.join(".github/workflows")).unwrap();
        fs::create_dir_all(test_dir.join("target")).unwrap();
        fs::write(test_dir.join(".gitignore"), "target/\n").unwrap();
        fs::write(test_dir.join(".git/HEAD"), "ref").unwrap();
        fs::write(test_dir.join(".git/objects/x"), "blob").unwrap();
        fs::write(test_dir.join(".github/workflows/ci.yml"), "on: push").unwrap();
        fs::write(test_dir.join("target/blob"), "built").unwrap();
        fs::write(test_dir.join("a.txt"), "a").unwrap();

        let files = list_files(&test_dir).expect("list_files should succeed");
        let names: Vec<String> = files
            .iter()
            .map(|f| {
                f.trim_start_matches(&*test_dir.to_string_lossy())
                    .trim_start_matches(['/', '\\'])
                    .replace('\\', "/")
            })
            .collect();

        assert!(names.contains(&"a.txt".to_string()));
        assert!(names.contains(&".gitignore".to_string()));
        assert!(names.contains(&".github/workflows/ci.yml".to_string()));
        assert!(!names.iter().any(|n| n.starts_with(".git/")), "{names:?}");
        assert!(!names.iter().any(|n| n.starts_with("target/")), "{names:?}");
    }

    #[test]
    fn list_files_errors_when_path_is_not_directory() {
        let file_path_tmp = tempfile::tempdir().unwrap();
        let file_path = file_path_tmp
            .path()
            .join("spark_test_list_files_not_dir.txt");
        fs::write(&file_path, "x").unwrap();

        let err = list_files(&file_path).expect_err("should fail for non-directory");
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }
}
