use crate::errors::BuildError;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn clean_dir(directory: &PathBuf) -> Result<(), BuildError> {
    if !fs::exists(directory).map_err(|source| BuildError::CheckPathExists {
        path: directory.clone(),
        source,
    })? {
        return Ok(());
    }

    fs::remove_dir_all(directory).map_err(|source| BuildError::CleanDirectory {
        path: directory.clone(),
        source,
    })?;
    Ok(())
}

pub fn ensure_dir(directory: &PathBuf) -> Result<(), BuildError> {
    fs::create_dir(directory).map_err(|source| BuildError::CreateDirectory {
        path: directory.clone(),
        source,
    })?;
    Ok(())
}

pub fn write_file(path: &Path, content: &str) -> Result<(), BuildError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| BuildError::CreateDirectory {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let mut file = fs::File::create(path).map_err(|source| BuildError::WriteFile {
        path: path.to_path_buf(),
        source,
    })?;
    file.write_all(content.as_bytes())
        .map_err(|source| BuildError::WriteFile {
            path: path.to_path_buf(),
            source,
        })?;
    Ok(())
}

pub fn copy_dir_contents(
    src: impl AsRef<Path>,
    dest: impl AsRef<Path>,
) -> Result<usize, BuildError> {
    let src = src.as_ref();
    let dest = dest.as_ref();
    let mut copied = 0;

    for entry in fs::read_dir(src).map_err(|source| BuildError::ReadDirectory {
        path: src.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| BuildError::ReadDirectoryEntry {
            directory: src.to_path_buf(),
            source,
        })?;
        let ty = entry
            .file_type()
            .map_err(|source| BuildError::ReadDirectoryEntry {
                directory: src.to_path_buf(),
                source,
            })?;
        if ty.is_dir() {
            //TODO: Update so that files can be referenced and copied recursively
            continue;
        } else {
            let source_path = entry.path();
            let destination_path = dest.join(entry.file_name());
            fs::copy(&source_path, &destination_path).map_err(|source| BuildError::CopyFile {
                source_path,
                destination_path,
                source,
            })?;
            copied += 1;
        }
    }

    Ok(copied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_dir(name: &str) -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("cable-{name}-{id}"))
    }

    #[test]
    fn write_file_creates_parent_directories() {
        let root = test_dir("write-file");
        let path = root.join("posts").join("hello.html");

        write_file(&path, "hello").unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "hello");
        clean_dir(&root).unwrap();
    }

    #[test]
    fn copy_dir_contents_copies_only_files() {
        let root = test_dir("copy-dir");
        let src = root.join("src");
        let dest = root.join("dest");
        fs::create_dir_all(src.join("nested")).unwrap();
        fs::create_dir_all(&dest).unwrap();
        fs::write(src.join("one.txt"), "one").unwrap();
        fs::write(src.join("nested").join("two.txt"), "two").unwrap();

        let copied = copy_dir_contents(&src, &dest).unwrap();

        assert_eq!(copied, 1);
        assert_eq!(fs::read_to_string(dest.join("one.txt")).unwrap(), "one");
        assert!(!dest.join("nested").exists());
        clean_dir(&root).unwrap();
    }
}
