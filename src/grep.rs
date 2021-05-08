use anyhow::Result;
use ignore::Walk;
use std::fs;
use std::path::PathBuf;

use crate::import_string;
use crate::path;

pub fn find_affected_files(current_dir: &PathBuf, moved_file: &PathBuf) -> Result<Vec<PathBuf>> {
    let full_moved_path = path::join(current_dir, moved_file)?;

    let affected_files = Walk::new(current_dir)
        .into_iter()
        .filter_map(|result| result.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|file_name| file_name.ends_with(".ts") || file_name.ends_with(".tsx"))
                .unwrap_or(false)
        })
        .map(|entry| entry.path().to_path_buf())
        .filter(|affected_file| !affected_file.eq(&full_moved_path))
        .filter(|affected_file| {
            has_import_to_file(&affected_file, &full_moved_path).unwrap_or(false)
        })
        .collect();

    Ok(affected_files)
}

fn has_import_to_file(source_file: &PathBuf, imported_file: &PathBuf) -> Result<bool> {
    let import_string = import_string::from_paths(&source_file, &imported_file)?;
    let content = fs::read_to_string(source_file)?;
    Ok(content.contains(&import_string))
}
