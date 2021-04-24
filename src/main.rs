use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::vec;

use ropey::Rope;

mod fault;
mod query;

use crate::fault::Fault;
use crate::query::{query_imports, Lang};

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        0 | 1 => println!("Gimme file"),
        2 => println!("Give new file path"),
        _ => mvts(&args[1], &args[2]).expect("fail"),
    }
}

fn infer_langauge_from_suffix(file_name: &String) -> Lang {
    let suffix = file_name
        .split('.')
        .last()
        .expect("Can't infer file type from file name");

    match suffix {
        "ts" => Lang::Ts,
        "tsx" => Lang::Tsx,
        _ => panic!("Expected .ts or .tsx file"),
    }
}

fn get_relative_imports(
    text: &Rope,
    locations: &Vec<(tree_sitter::Point, tree_sitter::Point)>,
) -> Vec<(usize, usize, String)> {
    locations
        .into_iter()
        .map(|(start_point, end_point)| {
            let start_idx = text.line_to_char(start_point.row) + start_point.column + 1;
            let end_idx = text.line_to_char(end_point.row) + end_point.column - 1;
            let import_string = text.slice(start_idx..end_idx).to_string();

            (start_idx, end_idx, import_string)
        })
        .filter(|(_, _, import_string)| import_string.starts_with("."))
        .collect()
}

fn mvts(file_name: &String, target_path: &String) -> Result<(), Fault> {
    let source_code = fs::read_to_string(file_name).expect("Unable to read file");
    let language = infer_langauge_from_suffix(&file_name);

    let imports = query_imports(&source_code, language)?;
    let text = Rope::from_str(&source_code);

    let rel_imports = get_relative_imports(&text, &imports);

    let orig_file_dir = get_file_dir(&file_name)?;
    // let target_file_dir = get_file_dir(&target_path)?;

    rel_imports.into_iter().for_each(|(_, _, import_string)| {
        file_path_from_import_string(&orig_file_dir, &target_file_dir, &import_string)
    });

    Ok(())
}

fn get_file_dir(file_path: &String) -> Result<PathBuf, Fault> {
    let mut p = Path::new(file_path).to_path_buf();
    if p.is_file() {
        p.pop();
    }
    let canon_path = fs::canonicalize(p)?;
    Ok(canon_path)
}

fn file_path_from_import_string(
    orig_file_dir: &PathBuf,
    target_file_dir: &PathBuf,
    orig_import: &String,
) {
    let import_strings: Vec<_> = vec![".ts", ".tsx", ".js", ".jsx"]
        .into_iter()
        .filter_map(|suffix| {
            let path_string = orig_import.clone() + suffix;
            let path = Path::new(&path_string);

            let mut abs_path = orig_file_dir.clone();
            abs_path.push(path);

            fs::canonicalize(abs_path).ok()
        })
        .collect();

    for import_string in import_strings {
        println!("{:?}", import_string);
        let lol = diff_paths(import_string, target_file_dir);
        // let lol = diff_paths(target_file_dir, import_string);
        println!("{:?}", lol);
    }
}

// fn new_import_path(orig_file_dir: Path, new_file_dir: Path, orig_import: String) { }

// Copy pasted from
// https://github.com/Manishearth/pathdiff/blob/master/src/lib.rs
// That seems to be copy-pasted from
// https://github.com/rust-lang/rust/blob/e1d0de82cc40b666b88d4a6d2c9dcbc81d7ed27f/src/librustc_back/rpath.rs#L116-L158
// : D
pub fn diff_paths<P, B>(path: P, base: B) -> Option<PathBuf>
where
    P: AsRef<Path>,
    B: AsRef<Path>,
{
    let path = path.as_ref();
    let base = base.as_ref();

    if path.is_absolute() != base.is_absolute() {
        if path.is_absolute() {
            Some(PathBuf::from(path))
        } else {
            None
        }
    } else {
        let mut ita = path.components();
        let mut itb = base.components();
        let mut comps: Vec<Component> = vec![];
        loop {
            match (ita.next(), itb.next()) {
                (None, None) => break,
                (Some(a), None) => {
                    comps.push(a);
                    comps.extend(ita.by_ref());
                    break;
                }
                (None, _) => comps.push(Component::ParentDir),
                (Some(a), Some(b)) if comps.is_empty() && a == b => (),
                (Some(a), Some(b)) if b == Component::CurDir => comps.push(a),
                (Some(_), Some(b)) if b == Component::ParentDir => return None,
                (Some(a), Some(_)) => {
                    comps.push(Component::ParentDir);
                    for _ in itb {
                        comps.push(Component::ParentDir);
                    }
                    comps.push(a);
                    comps.extend(ita.by_ref());
                    break;
                }
            }
        }
        Some(comps.iter().map(|c| c.as_os_str()).collect())
    }
}

// Possible path calc:
// source_code_dir = get_dir(source-file)
// for each import_path:
//     import_path_dir = get_dir(import_path)
//     abs_path = join(source_code_dir, import_path_dir)
//
//     new_import_path = join(
//         get_relative_path(abs_path, target_path_dir),
//         get_file(import_path)
//     )
