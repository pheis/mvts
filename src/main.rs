use std::fs;
use std::path::{Component, Path, PathBuf};
use std::vec;

use anyhow::{anyhow, Context, Error, Result};
use ropey::Rope;
use structopt::StructOpt;
use tree_sitter::{Language, Parser, Point, Query, QueryCursor, Tree};

// mod error;
// mod parse;

// use crate::error::Error;
// use crate::parse::{query_imports, Lang};

extern "C" {
    fn tree_sitter_typescript() -> Language;
    fn tree_sitter_tsx() -> Language;
}

#[derive(StructOpt)]
struct Cli {
    /// The pattern to look for
    #[structopt(parse(from_os_str))]
    source: std::path::PathBuf,
    /// The path to the file to read
    #[structopt(parse(from_os_str))]
    target: std::path::PathBuf,
}

fn main() -> Result<(), Error> {
    let Cli { source, target } = Cli::from_args();

    let imports = parse_imports(&source)?;
    let paths = to_file_paths(&source, &imports)?;

    for p in paths {
        println!("{:?}", p);
    }

    Ok(())
}

fn infer_langauge_from_suffix(file_name: &PathBuf) -> Result<Language> {
    let suffix = file_name
        .extension()
        .and_then(|osStr| osStr.to_str())
        .ok_or(Error::msg("Missing suffix on file"))?;

    match suffix {
        "ts" => Ok(unsafe { tree_sitter_typescript() }),
        "tsx" => Ok(unsafe { tree_sitter_tsx() }),
        suffix => Err(anyhow!("{:?} files are not supported", suffix)),
    }
}

fn parse_treesitter_tree(source_code: &String, language: Language) -> Result<Tree> {
    let mut parser = Parser::new();
    parser
        .set_language(language)
        .map_err(|_| anyhow!("Language error"))?;
    parser
        .parse(source_code, None)
        .ok_or(anyhow!("Parser failure"))
}

struct Imports {
    indices: Vec<(usize, usize)>,
    text: Rope,
}

fn parse_imports(source_file: &PathBuf) -> Result<Imports> {
    let language = infer_langauge_from_suffix(&source_file)?;

    let source = fs::read_to_string(source_file)
        .with_context(|| format!("Cannot open file {:?}", &source_file))?;

    let tree = parse_treesitter_tree(&source, language)?;
    let root = tree.root_node();
    let query = Query::new(language, "(import_statement (string) @import)")
        .map_err(|_| anyhow!("Query failure"))?;
    let mut query_cursor = QueryCursor::new();

    let text = Rope::from_str(&source);
    let mut indices = vec![];

    // TODO what does callback do?
    for (query_matches, u) in query_cursor.captures(&query, root, |_| "") {
        let captures = query_matches.captures;
        for i in 0..(u + 1) {
            let node = captures[i].node;

            let start_point = node.start_position();
            let start_idx = text.line_to_char(start_point.row) + start_point.column + 1;

            let end_point = node.end_position();
            let end_idx = text.line_to_char(end_point.row) + end_point.column - 1;

            if text.slice(start_idx..end_idx).to_string().starts_with(".") {
                indices.push((start_idx, end_idx));
            }
        }
    }

    Ok(Imports { indices, text })
}

fn to_file_paths(source: &PathBuf, imports: &Imports) -> Result<Vec<PathBuf>> {
    imports
        .indices
        .iter()
        .map(|(start_idx, end_idx)| imports.text.slice(start_idx..end_idx))
        .map(|import| {
            let mut path = source.clone();
            path.pop();
            // path.push(import.to_string());

            let abs_path = vec![".ts", ".tsx", ".js", ".jsx", ".svg"]
                .into_iter()
                .flat_map(|suffix| {
                    let mut regular_file_path = path.clone();
                    regular_file_path.push(import.to_string() + suffix);

                    let mut index_file_path = path.clone();
                    index_file_path.push(import.to_string() + "/index" + suffix);

                    vec![regular_file_path, index_file_path].into_iter()
                })
                .find_map(|file_path| fs::canonicalize(file_path).ok());

            abs_path.ok_or(anyhow!("Could not resolve import {}", import.to_string()))
        })
        .collect()
}

//fn get_relative_imports(
//    text: &Rope,
//    locations: &Vec<(tree_sitter::Point, tree_sitter::Point)>,
//) -> Vec<(usize, usize, String)> {
//    locations
//        .into_iter()
//        .map(|(start_point, end_point)| {
//            let start_idx = text.line_to_char(start_point.row) + start_point.column + 1;
//            let end_idx = text.line_to_char(end_point.row) + end_point.column - 1;
//            let import_string = text.slice(start_idx..end_idx).to_string();

//            (start_idx, end_idx, import_string)
//        })
//        .filter(|(_, _, import_string)| import_string.starts_with("."))
//        .collect()
//}

//fn mvts(file_name: &String, target_path: &String) -> Result<(), Error> {
//    let source_code = fs::read_to_string(file_name).expect("Unable to read file");
//    let language = infer_langauge_from_suffix(&file_name);

//    let imports = query_imports(&source_code, language)?;
//    let text = Rope::from_str(&source_code);

//    let rel_imports = get_relative_imports(&text, &imports);

//    let orig_file_dir = get_file_dir(&file_name)?;
//    let target_file_dir = get_file_dir(&target_path)?;

//    rel_imports.into_iter().for_each(|(_, _, import_string)| {
//        file_path_from_import_string(&orig_file_dir, &target_file_dir, &import_string)
//    });

//    Ok(())
//}

//fn get_file_dir(file_path: &String) -> Result<PathBuf, Error> {
//    let mut p = Path::new(file_path).to_path_buf();
//    if p.is_file() {
//        p.pop();
//    }
//    let canon_path = fs::canonicalize(p)?;
//    Ok(canon_path)
//}

//fn file_path_from_import_string(
//    orig_file_dir: &PathBuf,
//    target_file_dir: &PathBuf,
//    orig_import: &String,
//) {
//    let import_strings: Vec<_> = vec![".ts", ".tsx", ".js", ".jsx"]
//        .into_iter()
//        .filter_map(|suffix| {
//            let path_string = orig_import.clone() + suffix;
//            let path = Path::new(&path_string);

//            let mut abs_path = orig_file_dir.clone();
//            abs_path.push(path);

//            fs::canonicalize(abs_path).ok()
//        })
//        .collect();

//    for import_string in import_strings {
//        println!("{:?}", import_string);
//        let lol = diff_paths(import_string, target_file_dir);
//        // let lol = diff_paths(target_file_dir, import_string);
//        println!("{:?}", lol);
//    }
//}

//// fn new_import_path(orig_file_dir: Path, new_file_dir: Path, orig_import: String) { }

//// Copy pasted from
//// https://github.com/Manishearth/pathdiff/blob/master/src/lib.rs
//// That seems to be copy-pasted from
//// https://github.com/rust-lang/rust/blob/e1d0de82cc40b666b88d4a6d2c9dcbc81d7ed27f/src/librustc_back/rpath.rs#L116-L158
//// : D
//pub fn diff_paths<P, B>(path: P, base: B) -> Option<PathBuf>
//where
//    P: AsRef<Path>,
//    B: AsRef<Path>,
//{
//    let path = path.as_ref();
//    let base = base.as_ref();

//    if path.is_absolute() != base.is_absolute() {
//        if path.is_absolute() {
//            Some(PathBuf::from(path))
//        } else {
//            None
//        }
//    } else {
//        let mut ita = path.components();
//        let mut itb = base.components();
//        let mut comps: Vec<Component> = vec![];
//        loop {
//            match (ita.next(), itb.next()) {
//                (None, None) => break,
//                (Some(a), None) => {
//                    comps.push(a);
//                    comps.extend(ita.by_ref());
//                    break;
//                }
//                (None, _) => comps.push(Component::ParentDir),
//                (Some(a), Some(b)) if comps.is_empty() && a == b => (),
//                (Some(a), Some(b)) if b == Component::CurDir => comps.push(a),
//                (Some(_), Some(b)) if b == Component::ParentDir => return None,
//                (Some(a), Some(_)) => {
//                    comps.push(Component::ParentDir);
//                    for _ in itb {
//                        comps.push(Component::ParentDir);
//                    }
//                    comps.push(a);
//                    comps.extend(ita.by_ref());
//                    break;
//                }
//            }
//        }
//        Some(comps.iter().map(|c| c.as_os_str()).collect())
//    }
//}

//// Possible path calc:
//// source_code_dir = get_dir(source-file)
//// for each import_path:
////     import_path_dir = get_dir(import_path)
////     abs_path = join(source_code_dir, import_path_dir)
////
////     new_import_path = join(
////         get_relative_path(abs_path, target_path_dir),
////         get_file(import_path)
////     )
