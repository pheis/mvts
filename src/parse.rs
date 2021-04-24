use std::fs;
use std::path::{Component, Path, PathBuf};

use ropey::Rope;
use tree_sitter::{Language, Parser, Point, Query, QueryCursor, Tree};

use crate::error::Error;

extern "C" {
    fn tree_sitter_typescript() -> Language;
    fn tree_sitter_tsx() -> Language;
}

pub fn parse_relative_imports(file_path: PathBuf) -> Result<(Vec<(usize, usize)>, Rope), Error> {
    let language =
    let source_code = fs::read_to_string(file_path);
}

fn infer_langauge_from_suffix(file_name: &String) -> Lang {
    let suffix = file_name
        .split('.')
        .last()
        .expect("Can't infer file type from file name");

    match suffix {
        "ts" => unsafe { tree_sitter_typescript() },
        "tsx" => unsafe { tree_sitter_tsx() },
        _ => panic!("Expected .ts or .tsx file"),
    }
}

// Parse_imports: (filepath) -> (Vec<(end, start)>, rope)

pub enum Lang {
    Tsx,
    Ts,
}

impl From<Lang> for Language {
    fn from(lang: Lang) -> Self {
        match lang {
            Lang::Ts => unsafe { tree_sitter_typescript() },
            Lang::Tsx => unsafe { tree_sitter_tsx() },
        }
    }
}

fn get_tree(source_code: &String, language: Language) -> Tree {
    let mut parser = Parser::new();
    parser.set_language(language).unwrap();
    parser.parse(source_code, None).unwrap()
}

pub fn query_imports(source_code: &String, lang: Lang) -> Result<Vec<(Point, Point)>, Error> {
    let language = lang.into();

    let tree = get_tree(source_code, language);
    let root_node = tree.root_node();

    let query = Query::new(language, "(import_statement (string) @import)")?;

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
    Ok(imports)
}
