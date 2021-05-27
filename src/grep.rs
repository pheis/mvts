use ignore::Walk;
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
