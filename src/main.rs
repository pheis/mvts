use std::env;
use std::fs;

use tree_sitter::{Language, Parser, TreeCursor};

extern "C" {
    fn tree_sitter_typescript() -> Language;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    // println!("{:?}", args);
    match args.len() {
        0 | 1 => println!("Gimme file"),
        _ => read_file(&args[1]),
    }

    // println!("{:?}", args[1]);
    // println!("juuh");

    // let language = unsafe { tree_sitter_typescript() };

    // let mut parser = Parser::new();
    // parser.set_language(language).unwrap();

    // let source_code = "console.log()";

    // let tree = parser.parse(source_code, None).unwrap();

    // let root_node = tree.root_node();

    // println!("{}", root_node.kind());
}

// get Language from suffix

fn infer_langauge_from_suffix(file_name: &String) -> Language {
    let suffix = file_name
        .split('.')
        .last()
        .expect("Can't infer file type from file name");

    match suffix {
        "ts" => Some(unsafe { tree_sitter_typescript() }),
        _ => None,
    }
    .expect("You shall not code other langs that Hypescript")
}

fn read_file(file_name: &String) -> () {
    let source_code = fs::read_to_string(file_name).expect("Unable to read file");
    let language = infer_langauge_from_suffix(&file_name);

    parse_program(source_code, language);
}

fn parse_program(source_code: String, language: Language) -> () {
    let mut parser = Parser::new();
    parser.set_language(language).unwrap();

    let tree = parser.parse(source_code, None).unwrap();

    let root_node = tree.root_node();

    let cursor = tree.walk();

    println!("{}", root_node.kind());
}

fn walk(cursor: TreeCursor) -> () {
    cursor.goto_first_child();

    let mut has_next = true;
    loop {
        let n = cursor.node();
        mut_has_next
    }
}
