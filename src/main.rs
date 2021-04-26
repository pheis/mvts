use std::fs;
use std::io::BufWriter;
use std::path::PathBuf;
use std::vec;

use anyhow::{anyhow, Context, Error, Result};
use pathdiff::diff_paths;
use regex::Regex;
use ropey::Rope;
use structopt::StructOpt;
use tree_sitter::{Language, Parser, Query, QueryCursor, Tree};
use tree_sitter_typescript::{language_tsx, language_typescript};
use walkdir::{DirEntry, WalkDir};

#[derive(StructOpt)]
struct Args {
    #[structopt(parse(from_os_str))]
    source: std::path::PathBuf,
    #[structopt(parse(from_os_str))]
    target: std::path::PathBuf,
}

fn main() -> Result<(), Error> {
    let args = Args::from_args();

    let text = update_imports(&args)?;

    // move_and_replace(&args, text)?;
    find_references(&args)?;

    Ok(())
}

fn move_and_replace(Args { target, source }: &Args, text: Rope) -> Result<()> {
    let mut target_path = target.clone();

    if target.is_dir() {
        let file_name = source.file_name().unwrap();
        target_path.push(file_name);
    }

    fs::rename(&source, &target_path)?;
    text.write_to(BufWriter::new(fs::File::create(target_path)?))?;

    Ok(())
}

fn get_canon_dir(path: &PathBuf) -> Result<PathBuf> {
    match path.is_dir() {
        true => Ok(fs::canonicalize(path)?),
        false => {
            let mut stem = path.clone();
            stem.pop();
            Ok(fs::canonicalize(stem)?)
        }
    }
}

fn filter_file(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| {
            let is_hidden_file = !s.eq(".") && s.starts_with(".");
            let is_node_module = s.eq("node_modules");
            !is_hidden_file & !is_node_module
        })
        .unwrap_or(false)
}

fn find_references(args: &Args) -> Result<()> {
    let canon_source_path = fs::canonicalize(&args.source)?;

    let walker = WalkDir::new(".")
        .into_iter()
        .filter_entry(|e| filter_file(e))
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|s| s.ends_with(".ts") || s.ends_with(".tsx"))
                .unwrap_or(false)
        })
        .filter(|entry| {
            fs::canonicalize(entry.path())
                .map_err(|_| anyhow!("no canon for you"))
                .and_then(|canon_path| sniff_ref_for_file(&canon_source_path, &canon_path))
                .unwrap_or(false)
        });

    for entry in walker {
        println!("{}", entry.path().display());
    }

    Ok(())
}

fn sniff_ref_for_file(source: &PathBuf, path: &PathBuf) -> Result<bool> {
    let import_string = get_ts_import(&source, &path)?;
    let content = fs::read_to_string(path)?;
    Ok(content.contains(&import_string))
}

fn infer_langauge_from_suffix(file_name: &PathBuf) -> Result<Language> {
    let suffix = file_name
        .extension()
        .and_then(|os_str| os_str.to_str())
        .ok_or(Error::msg("Missing suffix on file"))?;

    match suffix {
        "ts" => Ok(language_typescript()),
        "tsx" => Ok(language_tsx()),
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

fn update_imports(args: &Args) -> Result<Rope> {
    let language = infer_langauge_from_suffix(&args.source)?;

    let source = fs::read_to_string(&args.source)
        .with_context(|| format!("Cannot open file {:?}", &args.source))?;

    let tree = parse_treesitter_tree(&source, language)?;
    let root = tree.root_node();
    let query = Query::new(language, "(import_statement (string) @import)")
        .map_err(|_| anyhow!("Query failure"))?;
    let mut query_cursor = QueryCursor::new();

    let mut text = Rope::from_str(&source);

    // TODO what does callback do?
    for (query_matches, u) in query_cursor.captures(&query, root, |_| "") {
        let captures = query_matches.captures;
        for i in 0..(u + 1) {
            let node = captures[i].node;

            let start_point = node.start_position();
            let start_idx = text.line_to_char(start_point.row) + start_point.column + 1;

            let end_point = node.end_position();
            let end_idx = text.line_to_char(end_point.row) + end_point.column - 1;

            let import_string = text.slice(start_idx..end_idx).to_string();

            if !import_string.starts_with(".") {
                continue;
            }

            let new_import = update_import_string(args, &import_string)?;

            text.remove(start_idx..end_idx);
            text.insert(start_idx, &new_import);
        }
    }

    Ok(text)
}

fn update_import_string(Args { source, target }: &Args, import_string: &String) -> Result<String> {
    let mut source_dir =
        fs::canonicalize(source).with_context(|| format!("can't find source {:?}", source))?;
    source_dir.pop();

    let abs_import_path = vec![".ts", ".tsx", ".js", ".jsx", ".svg"]
        .into_iter()
        .flat_map(|suffix| {
            let mut non_index_file_path = source_dir.clone();
            non_index_file_path.push(import_string.clone() + suffix);

            let mut index_file_path = source_dir.clone();
            index_file_path.push(import_string.to_string() + "/index" + suffix);

            vec![non_index_file_path, index_file_path].into_iter()
        })
        .find_map(|file_path| fs::canonicalize(file_path).ok())
        .ok_or(anyhow!("Unable to resolve import {}", import_string))?;

    let target_dir = get_canon_dir(&target)?;

    let rel_import_string = diff_paths(&abs_import_path, &target_dir)
        .ok_or(anyhow!(
            "Cannot build relative import for {:?}",
            abs_import_path
        ))?
        .to_str()
        .ok_or(anyhow!("Malformed path {:?}", abs_import_path))?
        .to_string();

    Ok(to_typesript_import_string(&rel_import_string))
}

fn to_typesript_import_string(import_string: &String) -> String {
    let re = Regex::new(r"/index\.ts|\.\w+$").unwrap();
    let import_string = re.replace_all(import_string, "");

    let re = Regex::new(r"^index$").unwrap();
    let import_string = re.replace_all(&import_string, ".").to_string();

    match import_string.starts_with(".") {
        true => import_string,
        false => "./".to_owned() + &import_string,
    }
}

fn get_ts_import(from_file: &PathBuf, into_file: &PathBuf) -> Result<String> {
    let mut into_dir = into_file.clone();
    into_dir.pop();

    diff_paths(from_file, into_dir)
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .map(|s| to_typesript_import_string(&s))
        .ok_or(anyhow!(
            "Failed to get ts import {:?} {:?}",
            from_file,
            into_file,
        ))
}
