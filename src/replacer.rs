use anyhow::{anyhow, Result};
use ropey::Rope;
use tree_sitter::{Language, Parser, Query, QueryCursor, Tree};
use tree_sitter_typescript::{language_tsx, language_typescript};

const QUERY: &'static str = "(import_statement (string) @import)";

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

pub struct ImportReplacer {
    query: Query,
    tree: Tree,
    text: Rope,
}

impl ImportReplacer {
    pub fn new(source_code: &String, lang: Lang) -> Result<Self> {
        let language = to_language(&lang);

        Ok(ImportReplacer {
            tree: parse_treesitter_tree(&source_code, language)?,
            text: Rope::from_str(&source_code),
            query: Query::new(language, &QUERY).unwrap(),
        })
    }

    pub fn to_string(&self) -> String {
        self.text.to_string()
    }

    pub fn replace_imports<F>(&mut self, replacer: F) -> Result<()>
    where
        F: Fn(&String) -> Result<String>,
    {
        let root = self.tree.root_node();
        let mut query_cursor = QueryCursor::new();

        for (query_matches, u) in query_cursor.captures(&self.query, root, |_| "") {
            let captures = query_matches.captures;
            for i in 0..(u + 1) {
                let node = captures[i].node;

                let start_point = node.start_position();
                let start_idx = self.text.line_to_char(start_point.row) + start_point.column + 1;

                let end_point = node.end_position();
                let end_idx = self.text.line_to_char(end_point.row) + end_point.column - 1;

                let import_string = self.text.slice(start_idx..end_idx).to_string();

                if !import_string.starts_with(".") {
                    continue;
                }

                let new_import = replacer(&import_string)?;

                self.text.remove(start_idx..end_idx);
                self.text.insert(start_idx, &new_import);
            }
        }
        Ok(())
    }
}

fn parse_treesitter_tree(source_code: &String, language: Language) -> Result<Tree> {
    let mut parser = Parser::new();
    parser
        .set_language(language)
        .map_err(|_| anyhow!("Language error"))?;
    parser
        .parse(source_code, None)
        .ok_or(anyhow!("Failed to parse"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let source = r#"
            import some from '../../some';
            function main() {
                const other = require('./other');
            }
            "#
        .into();

        let mut import_replacer =
            super::ImportReplacer::new(&source, super::Lang::TypeScript).unwrap();

        import_replacer
            .replace_imports(|_| Ok("WORKS".into()))
            .unwrap();

        let new_source_code = import_replacer.text.to_string();

        assert!(new_source_code.contains("import some from 'WORKS';"));
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
