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
    // println!("{:?}", args);
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

    let imports = get_top_level_imports(source_code.clone(), language);

    let text = Rope::from_str(&source_code);

    for (sp, ep) in imports {
        let start_idx = text.line_to_char(sp.row) + sp.column;
        let end_idx = text.line_to_char(ep.row) + ep.column;

        println!("{}", text.slice(start_idx..end_idx));
    }
}

fn get_top_level_imports(source_code: String, language: Language) -> Vec<(Point, Point)> {
    let mut parser = Parser::new();
    parser.set_language(language).unwrap();

    let tree = parser.parse(source_code, None).unwrap();

    let root_node = tree.root_node();
    let mut cursor = tree.walk();

    let mut imports = vec![];

    for node in root_node
        .children(&mut cursor)
        .filter(|node| node.kind() == "import_statement")
    {
        let mut child_cursor = node.walk();
        for child in node
            .children(&mut child_cursor)
            .filter(|child| child.kind() == "string" && !child.is_extra())
        {
            let sp = child.start_position();
            let ep = child.end_position();
            imports.push((sp.clone(), ep.clone()))
        }
    }
    imports
}
