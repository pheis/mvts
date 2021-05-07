use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

use crate::import_string;
use crate::path;

fn is_ok_file(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| {
            let is_hidden_file = !s.eq(".") && s.starts_with('.');
            let is_node_module = s.eq("node_modules");
            !is_hidden_file & !is_node_module
        })
        .unwrap_or(false)
}

pub fn find_affected_files(current_dir: &PathBuf, moved_file: &PathBuf) -> Result<Vec<PathBuf>> {
    let full_moved_path = path::join(current_dir, moved_file)?;

    let walker = WalkDir::new(".")
        .into_iter()
        .filter_entry(|e| is_ok_file(e))
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|s| s.ends_with(".ts") || s.ends_with(".tsx"))
                .unwrap_or(false)
        })
        .filter(|entry| {
            let full_path = current_dir.join(entry.path());
            path::normalize(&full_path)
                .map_err(|_| anyhow!("Failed to normalize path {:?}", entry.path()))
                .and_then(|file_path| has_import_to_file(&file_path, &full_moved_path))
                .unwrap_or(false)
        });

    let mut files = vec![];

    for entry in walker {
        let full_entry_path = current_dir.join(entry.path());
        let full_entry_path = path::normalize(&full_entry_path)?;

        if full_entry_path.eq(&full_moved_path) {
            continue;
        }

        files.push(entry.path().to_path_buf());
    }

    Ok(files)
}

fn has_import_to_file(source_file: &PathBuf, imported_file: &PathBuf) -> Result<bool> {
    let import_string = import_string::from_paths(&source_file, &imported_file)?;
    let content = fs::read_to_string(source_file)?;
    Ok(content.contains(&import_string))
}
