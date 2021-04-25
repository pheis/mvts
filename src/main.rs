use std::fs;
use std::path::PathBuf;
use std::vec;

use anyhow::{anyhow, Context, Error, Result};
use pathdiff::diff_paths;
use regex::Regex;
use ropey::Rope;
use structopt::StructOpt;
use tree_sitter::{Language, Parser, Query, QueryCursor, Tree};

extern "C" {
    fn tree_sitter_typescript() -> Language;
    fn tree_sitter_tsx() -> Language;
}

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    source: std::path::PathBuf,
    #[structopt(parse(from_os_str))]
    target: std::path::PathBuf,
}

fn main() -> Result<(), Error> {
    let Cli { source, target } = Cli::from_args();

    let imports = parse_imports(&source)?;
    let paths = to_file_paths(&source, &imports)?;

    let absolute_target_path = fs::canonicalize(&target)?;

    for import_path in paths {
        let relative_path = diff_paths(&import_path, &absolute_target_path).ok_or(anyhow!(
            "Cannot build relative import for {:?}",
            import_path
        ))?;

        let path_string = relative_path
            .to_str()
            .ok_or(anyhow!("Import malformed path: {:?}", import_path))?;

        let lol = path_string.to_string();
        println!("{}", to_typesript_import_string(&lol));
    }

    Ok(())
}

fn infer_langauge_from_suffix(file_name: &PathBuf) -> Result<Language> {
    let suffix = file_name
        .extension()
        .and_then(|os_str| os_str.to_str())
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

fn to_typesript_import_string(import_string: &String) -> String {
    let re = Regex::new(r"/index\.ts|\.\w+$").unwrap();
    let import_string = re.replace_all(import_string, "");

    let re = Regex::new(r"^index$").unwrap();
    re.replace_all(&import_string, ".").to_string()
}
