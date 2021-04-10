use std::array;
use std::env;
use std::fs;
use std::vec;

use ropey::Rope;
use tree_sitter::{Language, Parser, Point, Query, QueryCursor, Tree, TreeCursor};

extern "C" {
    fn tree_sitter_typescript() -> Language;
    fn tree_sitter_tsx() -> Language;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        0 | 1 => println!("Gimme file"),
        _ => read_file(&args[1]),
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

fn read_file(file_name: &String) -> () {
    let source_code = fs::read_to_string(file_name).expect("Unable to read file");
    let language = infer_langauge_from_suffix(&file_name);

    query_imports(source_code.clone(), language);

    let imports = query_imports(source_code.clone(), language);

    let text = Rope::from_str(&source_code);

    for (start_point, end_point) in imports {
        let start_idx = text.line_to_char(start_point.row) + start_point.column + 1;
        let end_idx = text.line_to_char(end_point.row) + end_point.column - 1;

        let source = text.slice(start_idx..end_idx);

        let source_string = source.to_string();

        if !source_string.starts_with(".") {
            continue;
        }
        println!("{}", source);
    }
}

fn get_tree(source_code: String, language: Language) -> Tree {
    let mut parser = Parser::new();
    parser.set_language(language).unwrap();
    parser.parse(source_code, None).unwrap()
}

fn query_imports(source_code: String, language: Language) -> Vec<(Point, Point)> {
    let tree = get_tree(source_code, language);
    let root_node = tree.root_node();

    let query = Query::new(language, "(import_statement (string) @import)").expect("bad query");

    let mut query_cursor = QueryCursor::new();

    let mut imports = vec![];

    // TODO what does callback do?
    for (query_matches, u) in query_cursor.captures(&query, root_node, |_| "What does this do?") {
        let captures = query_matches.captures;
        for i in 0..(u + 1) {
            let node = captures[i].node;
            let start_point = node.start_position();
            let end_point = node.end_position();
            imports.push((start_point.clone(), end_point.clone()))
        }
    }
    imports
}

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
