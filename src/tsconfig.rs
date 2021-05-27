use std::collections;
use std::fs;
use std::path;

use serde::{Deserialize, Serialize};
// use serde_json::Result;

#[derive(Serialize, Deserialize, Debug)]
struct CompilerOptions {
    paths: Option<collections::HashMap<String, Vec<String>>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TSConfig {
    extends: Option<String>,
    compiler_options: CompilerOptions,
}

pub fn find_ts_config(current_dir: &path::Path) -> Option<path::PathBuf> {
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

fn get_paths_from_config(path: &path::Path) -> Option<collections::HashMap<String, String>> {
    let config = fs::read_to_string(path).ok()?;
    println!("{:?}", config);
    let TSConfig {
        extends,
        compiler_options,
    } = serde_json::from_str(&config).ok()?;

    println!("{:?}", path);

    println!("{:?}", compiler_options);

    match (compiler_options.paths, extends) {
        (None, None) => None,
        (Some(paths), _) => Some(paths),
        (None, Some(another_config_file)) => {
            let another_config_file = path.parent()?.join(another_config_file);
            get_paths_from_config(&another_config_file)
        }
    }
}

pub fn read_ts_config(current_dir: &path::Path) -> Option<collections::HashMap<String, String>> {
    let ts_config_file = find_ts_config(current_dir)?;
    get_paths_from_config(&ts_config_file)
}
