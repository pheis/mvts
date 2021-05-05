use anyhow::{anyhow, Result};
use regex::Regex;
use std::path::PathBuf;

use crate::path;

pub fn to_path(file: &PathBuf, import_string: &String) -> Result<PathBuf> {
    let dir = path::get_parent(file);
    let import_path: PathBuf = import_string.into();
    let path = dir.join(import_path);

    path::normalize(&path)
}

pub fn from_relative_path(rel_path: &PathBuf) -> Result<String> {
    let import_string = rel_path.to_str().ok_or(anyhow!("Non utf-8 path"))?;

    let re = Regex::new(r"/index\.ts|\.\w+$").unwrap();
    let import_string = re.replace_all(import_string, "");

    let re = Regex::new(r"^index$").unwrap();
    let import_string = re.replace_all(&import_string, ".").to_string();

    Ok(match import_string.starts_with(".") {
        true => import_string,
        false => "./".to_owned() + &import_string,
    })
}

pub fn from_paths(file: &PathBuf, required_file: &PathBuf) -> Result<String> {
    let file_dir = path::get_parent(file);

    let rel_path = path::diff(&file_dir, required_file)?;
    from_relative_path(&rel_path)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    macro_rules! import_from_relative_path_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (path, expected) = $value;
                let path: PathBuf = path.into();
                let result = super::from_relative_path(&path).unwrap();
                assert_eq!(expected, result);
            }
        )*
        }
    }

    import_from_relative_path_tests! {
        rel_path_to_import_0: ("index.ts", "."),
        rel_path_to_import_1: ("index.js", "."),
        rel_path_to_import_3: ("index.jsx", "."),
        rel_path_to_import_4: ("index.tsx", "."),
        rel_path_to_import_5: ("juuh/elikkas/index.ts", "./juuh/elikkas"),
        rel_path_to_import_6: ("juuh/elikkas/joo.tsx", "./juuh/elikkas/joo"),
    }

    macro_rules! gets_import_from_paths_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (path, required_file, expected) = $value;
                let path: PathBuf = path.into();
                let required_file: PathBuf = required_file.into();

                let result = super::from_paths(&path, &required_file).unwrap();
                assert_eq!(expected, result);
            }
        )*
        }
    }

    gets_import_from_paths_tests! {
        gets_import_from_paths_0: ("src/views/some/Juuh.tsx", "src/store/index.ts", "../../store"),
        gets_import_from_paths_1: ("some/index.ts", "other/no/common", "../other/no/common"),
        gets_import_from_paths_2: ("index.ts", "deeper/in/path", "./deeper/in/path"),
        gets_import_from_paths_3: ("lol.ts", "index.ts", "."),
    }

    macro_rules! to_path_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (path, import_string, expected) = $value;

                let path: PathBuf = path.into();
                let import_sring: String = import_string.into();

                let result = super::to_path(&path, &import_sring).unwrap();

                let expected: PathBuf = expected.into();

                assert_eq!(expected, result);
            }
        )*
        }
    }

    to_path_tests! {
        to_path_0: ("a/b/c/file.ts", "../../other", "a/other"),
        to_path_1: ("a/b/c/file.ts", ".", "a/b/c"),
        to_path_2: ("a/b/c/file.ts", "./d/e", "a/b/c/d/e"),
        to_path_3: ("a/b/c/file.ts", "../", "a/b"),
    }
}
