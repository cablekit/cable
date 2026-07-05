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
