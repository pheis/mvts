use anyhow::{anyhow, Result};
use ropey::Rope;
use tree_sitter::{Language, Parser, Query, QueryCursor, Tree};
use tree_sitter_typescript::{language_tsx, language_typescript};

const QUERY: &str = "(import_statement (string) @import)";

pub enum Lang {
    TypeScript,
    TypeScriptTsx,
}

fn to_language(language: &Lang) -> Language {
    match language {
        Lang::TypeScript => language_typescript(),
        Lang::TypeScriptTsx => language_tsx(),
    }
}

pub struct CST {
    query: Query,
    tree: Tree,
    text: Rope,
}

impl CST {
    pub fn new(source_code: &str, lang: Lang) -> Result<Self> {
        let language = to_language(&lang);

        Ok(CST {
            tree: parse_treesitter_tree(source_code, language)?,
            text: Rope::from_str(&source_code),
            query: Query::new(language, &QUERY).unwrap(),
        })
    }

    pub fn get_source_code(&self) -> String {
        self.text.to_string()
    }

    // Iter<Node>,  no mut, split text out of this struct
    pub fn replace_all_imports<F>(&mut self, replacer: F) -> Result<()>
    where
        F: Fn(&String) -> Result<String>,
    {
        let root = self.tree.root_node();
        let mut query_cursor = QueryCursor::new();

        for node in query_cursor
            .matches(&self.query, root, |_| "")
            .into_iter()
            .flat_map(|qm| qm.captures.iter())
            .map(|qc| qc.node)
        {
            let start_point = node.start_position();
            let start_idx = self.text.line_to_char(start_point.row) + start_point.column + 1;

            let end_point = node.end_position();
            let end_idx = self.text.line_to_char(end_point.row) + end_point.column - 1;

            let import_string = self.text.slice(start_idx..end_idx).to_string();

            if !import_string.starts_with('.') {
                continue;
            }

            let new_import = replacer(&import_string)?;

            if new_import.eq(&import_string) {
                continue;
            }

            self.text.remove(start_idx..end_idx);
            self.text.insert(start_idx, &new_import);
        }
        Ok(())
    }

    pub fn replace_one_import(&mut self, old: &str, new: &str) -> Result<()> {
        self.replace_all_imports(|import_string| match import_string {
            x if x.eq(old) => Ok(new.to_string()),
            _ => Ok(import_string.clone()),
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_replaces_many_imports() {
        let source: String = r#"
            import some from '../../some';
            import other from '../../other';
            function main() {
                const other = require('./other');
            }
            "#
        .into();

        let mut concrete_syntax_tree = super::CST::new(&source, super::Lang::TypeScript).unwrap();

        concrete_syntax_tree
            .replace_all_imports(|_| Ok("WORKS".into()))
            .unwrap();

        let new_source_code = concrete_syntax_tree.get_source_code();

        assert!(new_source_code.contains("import some from 'WORKS';"));
        assert!(new_source_code.contains("import other from 'WORKS';"));
    }

    #[test]
    fn it_replaces_one_import() {
        let source: String = r#"
            import some from '../../some';
            import other from '../../other';
            function main() {
                const other = require('./other');
            }
            "#
        .into();

        let mut concrete_syntax_tree = super::CST::new(&source, super::Lang::TypeScript).unwrap();

        concrete_syntax_tree
            .replace_one_import("../../other", "replaced")
            .unwrap();

        let new_source_code = concrete_syntax_tree.get_source_code();

        assert!(new_source_code.contains("import some from '../../some';"));
        assert!(new_source_code.contains("import other from 'replaced';"));
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
