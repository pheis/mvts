use anyhow::{anyhow, Result};
use ropey::Rope;
use std::path::PathBuf;
use tree_sitter::{Language, Parser, Query, QueryCursor, Tree};
use tree_sitter_typescript::{language_tsx, language_typescript};

const QUERY: &str = "(import_statement (string) @import)(export_statement (string) @import)";

fn infer_langauge(file_name: &PathBuf) -> Result<Language> {
    let suffix = file_name
        .extension()
        .and_then(|os_str| os_str.to_str())
        .ok_or_else(|| anyhow!("Missing suffix on file"))?;

    match suffix {
        "ts" => Ok(language_typescript()),
        "tsx" => Ok(language_tsx()),
        suffix => Err(anyhow!("{:?} files are not supported", suffix)),
    }
}

#[derive(Debug)]
pub struct TextSlice {
    start_row: usize,
    start_col: usize,
    end_row: usize,
    end_col: usize,
}

impl TextSlice {
    pub fn to_index_range(&self, rope: &Rope) -> (usize, usize) {
        let start_idx = rope.line_to_char(self.start_row) + self.start_col;
        let end_idx = rope.line_to_char(self.end_row) + self.end_col;
        (start_idx, end_idx)
    }
}

pub struct ImportFinder {
    tree: Tree,
    query: Query,
    cursor: QueryCursor,
}

impl ImportFinder {
    pub fn new(source_code: &str, file_path: &PathBuf) -> Result<Self> {
        let language = infer_langauge(file_path)?;

        let tree = parse_treesitter_tree(source_code, language)?;
        let query = Query::new(language, &QUERY).unwrap();
        let cursor = QueryCursor::new();

        Ok(Self {
            tree,
            query,
            cursor,
        })
    }

    pub fn find_imports(&mut self) -> impl Iterator<Item = TextSlice> + '_ {
        self.cursor
            .matches(&self.query, self.tree.root_node(), |_| "")
            .into_iter()
            .flat_map(|qm| qm.captures.iter())
            .map(|query_capture| query_capture.node)
            .map(|node| {
                let start_point = node.start_position();
                let end_point = node.end_position();

                TextSlice {
                    start_row: start_point.row,
                    start_col: start_point.column + 1,
                    end_row: end_point.row,
                    end_col: end_point.column - 1,
                }
            })
    }
}

fn parse_treesitter_tree(source_code: &str, language: Language) -> Result<Tree> {
    let mut parser = Parser::new();
    parser
        .set_language(language)
        .map_err(|_| anyhow!("Language error"))?;
    parser
        .parse(source_code, None)
        .ok_or_else(|| anyhow!("Failed to parse"))
}

pub fn replace_imports<F>(path: &PathBuf, source_code: &str, replacer: F) -> Result<Option<Rope>>
where
    F: Fn(&String) -> Result<String>,
{
    let mut import_finder = ImportFinder::new(&source_code, path)?;
    let mut rope = Rope::from_str(&source_code);

    let mut has_mutated = false;

    for text_slice in import_finder.find_imports() {
        let (start_idx, end_idx) = text_slice.to_index_range(&rope);

        let old_import = rope.slice(start_idx..end_idx).to_string();

        if !old_import.starts_with('.') {
            continue;
        }

        let new_import = replacer(&old_import)?;

        if old_import.eq(&new_import) {
            continue;
        }

        has_mutated = true;
        rope.remove(start_idx..end_idx);
        rope.insert(start_idx, &new_import);
    }

    match has_mutated {
        true => Ok(Some(rope)),
        false => Ok(None),
    }
}

// (call_expression
//   (identifier) @constant
//   (#match? @constant "require")
//   (arguments (string) @lol)
// (call_expression
//   (identifier) @constant
//   (#match? @constant "require")
//   (arguments (string) @lol)
// )
// )
//
//
//
// (call_expression
//   (import)
//   (arguments (string) @lol)
// )
// (import_statement (string) @lol)
// (call_expression
//    (identifier) @constant
//    (#match? @constant "require")
//    (arguments (string) @lol)
// )
//
//
//
// (call_expression
// (identifier) @function_name
// (match? @function_name "require")
// (arguments (string) @import_string))
//
