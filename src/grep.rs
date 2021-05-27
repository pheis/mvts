use ignore::Walk;
use std::fs;
use std::path;
use std::path::PathBuf;

pub fn iter_files(dir: &PathBuf) -> impl Iterator<Item = PathBuf> {
    Walk::new(dir)
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
}

pub fn find_ts_config(current_dir: &path::Path) -> Option<PathBuf> {
    fs::read_dir(current_dir)
        .ok()?
        .find_map(|entry| {
            let entry = entry.ok()?;

            let file_name = entry.file_name();
            let file_name = file_name.to_str()?;

            match file_name {
                "tsconfig.json" => Some(entry.path()),
                _ => None,
            }
        })
        .or_else(|| {
            let parent_dir = current_dir.parent();

            match parent_dir {
                Some(dir) => find_ts_config(dir),
                None => None,
            }
        })
}
