use std::collections::HashMap;
use std::path::PathBuf;

pub struct Import {
    source_file: PathBuf,
    string: String,
}

pub struct ImportUpdater {
    tsconfig_paths: HashMap<String, String>,
    files: HashMap<PathBuf, PathBuf>,
}
