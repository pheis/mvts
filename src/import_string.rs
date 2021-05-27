use anyhow::{anyhow, Result};
use std::path::PathBuf;

use crate::path;

pub fn to_path(file: &PathBuf, import_string: &str) -> Result<PathBuf> {
    let dir = path::get_parent(file);
    let import_path: PathBuf = import_string.into();
    let path = dir.join(import_path);

    path::normalize(&path)
}

fn from_relative_path(rel_path: &PathBuf) -> Result<String> {
    let import_string = rel_path.to_str().ok_or_else(|| anyhow!("Non utf-8 path"))?;

    Ok(match import_string.as_ref() {
        x if x.starts_with('.') => x.into(),
        "" => ".".into(),
        _ => "./".to_owned() + &import_string,
    })
}

pub fn from_paths(file: &PathBuf, required_file: &PathBuf) -> Result<String> {
    let file_dir = path::get_parent(file);

    let rel_path = path::diff(&file_dir, required_file)?;
    from_relative_path(&rel_path)
}

pub fn to_node_import(import_sting: &str) -> &str {
    import_sting
        .strip_suffix("/index.ts")
        .or_else(|| import_sting.strip_suffix("/index.tsx"))
        .or_else(|| import_sting.strip_suffix("/index.js"))
        .or_else(|| import_sting.strip_suffix("/index.jsx"))
        .or_else(|| import_sting.strip_suffix("index.ts"))
        .or_else(|| import_sting.strip_suffix("index.tsx"))
        .or_else(|| import_sting.strip_suffix("index.js"))
        .or_else(|| import_sting.strip_suffix("index.jsx"))
        .or_else(|| import_sting.strip_suffix(".ts"))
        .or_else(|| import_sting.strip_suffix(".tsx"))
        .or_else(|| import_sting.strip_suffix(".js"))
        .or_else(|| import_sting.strip_suffix(".jsx"))
        .unwrap_or(import_sting)
}

pub fn is_import_from(
    source_file: &PathBuf,
    required_file: &PathBuf,
    import_string: &str,
) -> Result<bool> {
    let rel_string = from_paths(&source_file, &required_file)?;
    let wo_index = to_node_import(&rel_string);
    let with_index = wo_index.to_owned() + "/index";

    Ok(
        import_string.eq(&rel_string)
            || import_string.eq(wo_index)
            || import_string.eq(&with_index),
    )
}

pub struct SourceFileRename<'a> {
    pub old_location: &'a PathBuf,
    pub new_location: &'a PathBuf,
    pub import_string: &'a str,
}

pub fn rename_source_file(
    SourceFileRename {
        old_location,
        new_location,
        import_string,
    }: &SourceFileRename,
) -> Result<String> {
    let path = to_path(old_location, import_string)?;
    from_paths(new_location, &path)
}

pub struct RequiredFileRename<'a> {
    pub source_file: &'a PathBuf,
    pub import_string: &'a str,
    pub old_location: &'a PathBuf,
    pub new_location: &'a PathBuf,
}

pub fn rename_required_file(
    RequiredFileRename {
        source_file,
        import_string,
        old_location,
        new_location,
    }: &RequiredFileRename,
) -> Result<String> {
    if !is_import_from(source_file, old_location, import_string)? {
        return Ok(import_string.to_string());
    }

    let suffix: PathBuf = import_string.into();
    let suffix = suffix.extension();

    let new_import_string = from_paths(source_file, new_location)?;

    Ok(match suffix {
        Some(_) => new_import_string,
        None => to_node_import(&new_import_string).to_string(),
    })
}

// pub fn update_import(
//     file_path: &PathBuf,
//     import_string: &str,
//     source_path: &PathBuf,
//     target_path: &PathBuf,
// ) -> String {
// }

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    macro_rules! to_node_import_tests{
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (input, expected) = $value;


                let result = super::to_node_import(input);
                assert_eq!(result, expected);
            }
        )*
        }
    }

    to_node_import_tests! {
        to_node_import_test_00: ("./a/b.ts", "./a/b"),
        to_node_import_test_01: ("./a/b.tsx", "./a/b"),
        to_node_import_test_02: ("./a/b.js", "./a/b"),
        to_node_import_test_03: ("./a/b.jsx", "./a/b"),
        to_node_import_test_04: ("./a/b.svg", "./a/b.svg"),
        to_node_import_test_05: ("./a/index.svg", "./a/index.svg"),

        to_node_import_test_06: ("./a/index.ts", "./a"),
        to_node_import_test_07: ("./a/b/index.jsx", "./a/b"),
        to_node_import_test_08: ("../b/index.jsx", "../b"),
    }

    macro_rules! rename_source_file_tests{
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (old_file, new_file, import_string, expected) = $value;
                let old_file: PathBuf = old_file.into();
                let new_file: PathBuf = new_file.into();

                let source_file_rename = super::SourceFileRename {
                    old_location: &old_file,
                    new_location: &new_file,
                    import_string,
                };

                let result = super::rename_source_file(&source_file_rename).unwrap();
                assert_eq!(result, expected);
            }
        )*
        }
    }

    rename_source_file_tests! {
        rename_to_new_file_00: ("a/old.ts", "a/new.ts", "./index",  "./index"),
        rename_to_new_file_01: ("a/old.ts", "a/new.ts", "./index.ts",  "./index.ts"),
        rename_to_new_file_02: ("a/old.ts", "a/new.ts", "./some.svg",  "./some.svg"),
        rename_to_new_file_04: ("a/old.ts", "a/b/new.ts", "./mod.ts",  "../mod.ts"),
        rename_to_new_file_05: ("a/old.ts", "a/new.ts", "./x/y/z/index",  "./x/y/z/index"),
        rename_to_new_file_06: ("a/old.ts", "a/new.ts", "./x/y/z/index.ts",  "./x/y/z/index.ts"),
        rename_to_new_file_07: ("a/b/c/old.ts", "a/x/y/z/new.ts", "./s/t/u/file",  "../../../b/c/s/t/u/file"),
        rename_to_new_file_08: ("a/b/c/old.ts", "a/x/y/z/new.ts", "./s/t/u/file.jsx",  "../../../b/c/s/t/u/file.jsx"),
        rename_to_new_file_09: ("a/b/c/old.ts", "a/x/y/z/new.ts", "./s/t/u/file.svg",  "../../../b/c/s/t/u/file.svg"),
        rename_to_new_file_10: ("a/b/c/old.ts", "a/x/y/z/new.ts", "./s/t/u/index.ts",  "../../../b/c/s/t/u/index.ts"),
    }

    macro_rules! rename_required_file_tests{
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (source_file, import_string, old_location, new_location, expected) = $value;
                let source_file: PathBuf = source_file.into();
                let old_location: PathBuf = old_location.into();
                let new_location: PathBuf = new_location.into();

                let required_file_rename = super::RequiredFileRename {
                    source_file: &source_file,
                    import_string,
                    old_location: &old_location,
                    new_location: &new_location,
                };

                let result = super::rename_required_file(&required_file_rename).unwrap();
                assert_eq!(result, expected);
            }
        )*
        }
    }

    rename_required_file_tests! {
        rename_req_file_00: ("a/source.ts", "./b", "a/b/index.ts",  "a/b/c/index.ts", "./b/c"),
        rename_req_file_01: ("a/source.ts", "./b.ts", "a/b/index.ts",  "a/b/c/index.ts", "./b.ts"), // stays same
        rename_req_file_02: ("a/source.ts", "./b.ts", "a/b.ts",  "a/b/c/index.ts", "./b/c/index.ts"), // deno move
        rename_req_file_03: ("a/b/source.ts", "..", "a/index.ts",  "a/b/c/index.ts", "./c"), // node move
    }

    macro_rules! is_relative_import_to_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (source_file, required_file, import_string, expected) = $value;
                let source_file: PathBuf = source_file.into();
                let required_file: PathBuf = required_file.into();

                let result = super::is_import_from(&source_file, &required_file, import_string,).unwrap();
                assert_eq!(result, expected);
            }
        )*
        }
    }

    is_relative_import_to_tests! {
        is_relative_import_to_0: ("a/some_file.ts", "a/index.ts", "./index",  true),
        is_relative_import_to_1: ("a/some_file.ts", "a/index.ts", ".", true),
        is_relative_import_to_2: ("a/b/some_file.ts", "a/index.ts", "..", true),
        is_relative_import_to_3: ("a/b/some_file.ts", "a/index.ts", "../index.ts", true),
        is_relative_import_to_4: ("a/b/some_file.ts", "a/c/some.svg", "../c/some.svg", true),
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
        gets_import_from_paths_0: ("src/views/some/Juuh.tsx", "src/store/index.ts", "../../store/index.ts"),
        gets_import_from_paths_1: ("some/index.ts", "other/no/common", "../other/no/common"),
        gets_import_from_paths_2: ("index.ts", "deeper/in/path", "./deeper/in/path"),
        gets_import_from_paths_3: ("lol.ts", "index.ts", "./index.ts"),
        gets_import_from_paths_4: ("a/b/c/index.ts", "a/b/c/d/index.tsx", "./d/index.tsx"),
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
        to_path_4: ("a/b/c/file.ts", "./index", "a/b/c/index"),
    }
}
