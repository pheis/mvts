use anyhow::{anyhow, Result};
use pathdiff::diff_paths;
use std::path::{Component, Path, PathBuf};
use std::vec;

pub fn normalize(path: &PathBuf) -> Result<PathBuf> {
    let mut skip = 0;
    let mut components = vec![];

    for component in path.components().into_iter().rev() {
        match component {
            Component::Normal(_) => {
                if skip > 0 {
                    skip -= 1;
                } else {
                    components.push(component);
                }
            }
            Component::CurDir => (),
            Component::ParentDir => {
                skip += 1;
            }
            _ => {
                components.push(component);
            }
        }
    }
    match skip {
        0 => Ok(components.into_iter().rev().collect()),
        _ => Err(anyhow!("Failed to normalize path {:?}", path)),
    }
}

pub fn diff(from_path: &PathBuf, to_path: &PathBuf) -> Result<PathBuf> {
    let normalized_from_path = normalize(from_path)?;
    let normalized_to_path = normalize(to_path)?;

    diff_paths(normalized_to_path, normalized_from_path).ok_or_else(|| {
        anyhow!(
            "Failed to get relative path from {:?} to {:?}",
            from_path,
            to_path,
        )
    })
}

pub fn get_parent(path: &PathBuf) -> PathBuf {
    let mut path = path.clone();
    path.pop();
    path
}

pub fn join(dir: &PathBuf, path: &PathBuf) -> Result<PathBuf> {
    let full_path = dir.join(path);
    normalize(&full_path)
}

pub fn move_path(file_path: &Path, source_path: &Path, target_path: &Path) -> Option<PathBuf> {
    match file_path.starts_with(source_path) {
        false => None,
        true => {
            let suffix = file_path.strip_prefix(source_path).ok()?;
            Some(target_path.join(suffix))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    macro_rules! move_path_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (file, source, target, expected) = $value;
                let file: PathBuf = file.into();
                let source: PathBuf = source.into();
                let target: PathBuf = target.into();

                let expected: Option<PathBuf> = expected.map(|s: &str| s.into());


                let result = super::move_path(&file, &source, &target);
                assert_eq!(result, expected);
            }
        )*
        }
    }

    move_path_tests! {
        move_0: ("/a/b/c", "/a/b", "/a/z", Some("/a/z/c")),
        move_1: ("a/b/c", "a/b", "a/z", Some("a/z/c")),
        move_2: ("x/b/c", "a/b", "a/z", None),
        move_3: ("a/b/c/d", "a", "z", Some("z/b/c/d")),
        move_4: ("src/some/juuh.ts", "src/other/juuh.ts", "src/other/elikkas.ts", None),
        move_5: ("src/some/juuh.ts", "src/some", "src/other", Some("src/other/juuh.ts")),
    }

    macro_rules! normalize_path_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (path, expected) = $value;
                let expected: PathBuf = expected.into();
                let path: PathBuf = path.into();
                let result = super::normalize(&path).unwrap();
                assert_eq!(expected, result);
            }
        )*
        }
    }

    normalize_path_tests! {
        strips_0: ("a/b/../c", "a/c"),
        strips_1: ("a/b/./../c", "a/c"),
        strips_2: ("a/b/./../c/d", "a/c/d"),
        strips_3: ("a/b/./../c/d/e/../f", "a/c/d/f"),
        strips_4: ("/a/b/./../c/d/e/../f", "/a/c/d/f"),
        strips_5: ("/a/b/./../c/d/e/../f.svg", "/a/c/d/f.svg"),
    }
}
