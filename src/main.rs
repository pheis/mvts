use std::env;
use std::fs;

use tree_sitter::{Language, Parser, Query, QueryCursor, Tree, TreeCursor};

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
}

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
    // get_top_level_imports(tree);
    list_imports(tree);
    // query_magic(tree, language);

    // let root_node = tree.root_node();

    // let mut cursor = tree.walk();
    // cursor.goto_first_child();
    // println!("{}", cursor.node().kind());

    // println!("{}", root_node.kind());

    // let q = Query::new(language, "(import_statement :source)").expect("lol");
    // let mut qc = QueryCursor::new();
    // let lol = qc.matches(&q, root_node, |node| node.kind());
    // for item in lol {
    //     for x in item.captures {
    //         println!("{}", x.node.kind());
    //     }
    // }
}

fn get_top_level_imports(tree: Tree) {
    let mut cursor = tree.walk();
    cursor.goto_first_child();
    loop {
        let node = cursor.node();
        if node.kind() == "import_statement" {
            for source_node in node
                .children(&mut node.walk())
                .filter(|child_node| child_node.kind() == "string")
            {
                println!("{}", source_node.child_count());
                println!("{}", source_node.is_named());
            }
        }

        let has_sibling = cursor.goto_next_sibling();
        if !has_sibling {
            break;
        }
    }
}

fn list_imports(tree: Tree) {
    let root_node = tree.root_node();
    let mut cursor = tree.walk();

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
            println!("{}", child.kind());
            println!("{}", sp);
            println!("{}", ep);
        }
    }
}
