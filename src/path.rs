use anyhow::{anyhow, Result};
use pathdiff::diff_paths;
use std::path::{Component, PathBuf};
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

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
