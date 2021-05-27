use std::collections::HashMap;
use std::path::PathBuf;

use crate::tsconfig;

pub struct Import {
    source_file: PathBuf,
    string: String,
}

pub struct ImportUpdater {
    tsconfig_paths: tsconfig::PathMap,
    files: HashMap<PathBuf, PathBuf>,
}
