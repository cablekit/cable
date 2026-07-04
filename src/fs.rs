use std::error::Error;
use std::path::{Path, PathBuf};
use std::fs;


pub fn clean_dir(directory: &PathBuf) -> Result<(), Box<dyn Error>>{
    if !fs::exists(directory)?{
        return Ok(())
    }

    fs::remove_dir_all(directory)?;
    Ok(())
}

pub fn ensure_dir(directory: &PathBuf) -> Result<(), Box<dyn Error>>{
    fs::create_dir(directory)?;
    Ok(())
}



pub fn copy_dir_contents(src: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<usize, Box<dyn Error>>{
    let mut copied = 0;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            //TODO: Update so that files can be referenced and copied recursively
            continue;
        } else {
            fs::copy(entry.path(), dest.as_ref().join(entry.file_name()))?;
            copied += 1;
        }
    }

    Ok(copied)
}
