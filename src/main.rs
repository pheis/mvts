use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::vec;

use ropey::Rope;
use tree_sitter::{Language, Parser, Tree};

mod fault;
mod query;

use crate::fault::Fault;
use crate::query::query_imports;

extern "C" {
    fn tree_sitter_typescript() -> Language;
    fn tree_sitter_tsx() -> Language;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        0 | 1 => println!("Gimme file"),
        2 => println!("Give new file path"),
        _ => mvts(&args[1], &args[2]).expect("fail"),
    }
}

fn infer_langauge_from_suffix(file_name: &String) -> Language {
    let suffix = file_name
        .split('.')
        .last()
        .expect("Can't infer file type from file name");

    match suffix {
        "ts" => unsafe { tree_sitter_typescript() },
        "tsx" => unsafe { tree_sitter_tsx() },
        _ => panic!("Expected .ts file"),
    }
}

fn mvts(file_name: &String, target_path: &String) -> Result<(), Fault> {
    let source_code = fs::read_to_string(file_name).expect("Unable to read file");
    let language = infer_langauge_from_suffix(&file_name);

    get_file_dir(file_name);
    let diff = diff_paths(file_name, target_path).expect("diff failure");

    println!("{:?}", diff);

    let imports = query_imports(source_code.clone(), language)?;

    let text = Rope::from_str(&source_code);

    for (start_point, end_point) in imports {
        let start_idx = text.line_to_char(start_point.row) + start_point.column + 1;
        let end_idx = text.line_to_char(end_point.row) + end_point.column - 1;

        let source = text.slice(start_idx..end_idx);

        let source_string = source.to_string();

        if !source_string.starts_with(".") {
            continue;
        }

        // let asdf = diff_paths(target_path, source_string.clone()).expect("oh noes we die here");

        // let orig_dir = get_file_dir(file_name);
        // let target_dir = get_file_dir(target_path);

        //let pdiff = diff_paths(orig_dir, target_dir).expect("blurbs");
        // let pdiff = diff_paths(target_dir, orig_dir).expect("blurbs");
        // println!("pdiff: {:?}", pdiff);

        // let qwer = diff_paths(source_string.clone(), pdiff).expect("oh no");
        // let qwer = diff_paths(pdiff, source_string.clone()).expect("oh no");

        let orig_dir = get_file_dir(file_name);

        let current_dir = env::current_dir().expect("lol wut");

        let abs_orig_dir = current_dir.join(orig_dir);

        let mut tf = Path::new(&source_string).to_path_buf();
        tf.pop();

        let p = abs_orig_dir.join(tf).canonicalize();

        println!("");
        println!("original: {}", source);
        println!("fp: {:?}", p);
        // println!("mutated: {:?}", qwer);
        println!("");
    }
    Ok(())
}

fn get_file_dir(file_path: &String) -> PathBuf {
    let mut p = Path::new(file_path).to_path_buf();
    if p.is_file() {
        p.pop();
    }
    p
}

fn get_tree(source_code: String, language: Language) -> Tree {
    let mut parser = Parser::new();
    parser.set_language(language).unwrap();
    parser.parse(source_code, None).unwrap()
}

// fn query_imports(source_code: String, language: Language) -> Result<Vec<(Point, Point)>, Fault> {
//     let tree = get_tree(source_code, language);
//     let root_node = tree.root_node();

//     let query = Query::new(language, "(import_statement (string) @import)")?;

//     let mut query_cursor = QueryCursor::new();

//     let mut imports = vec![];

//     // TODO what does callback do?
//     for (query_matches, u) in query_cursor.captures(&query, root_node, |_| "What does this do?") {
//         let captures = query_matches.captures;
//         for i in 0..(u + 1) {
//             let node = captures[i].node;
//             let start_point = node.start_position();
//             let end_point = node.end_position();
//             imports.push((start_point.clone(), end_point.clone()))
//         }
//     }
//     Ok(imports)
// }

// fn get_top_level_imports(source_code: String, language: Language) -> Vec<(Point, Point)> {
//     let tree = get_tree(source_code, language);

//     let root_node = tree.root_node();
//     let mut cursor = tree.walk();

//     let mut imports = vec![];

//     for node in root_node
//         .children(&mut cursor)
//         .filter(|node| node.kind() == "import_statement")
//     {
//         let mut child_cursor = node.walk();
//         for child in node
//             .children(&mut child_cursor)
//             .filter(|child| child.kind() == "string" && !child.is_extra())
//         {
//             let start_point = child.start_position();
//             let end_point = child.end_position();
//             imports.push((start_point.clone(), end_point.clone()))
//         }
//     }
//     imports
// }

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
