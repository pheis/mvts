use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

mod grep;
mod import_string;
mod parser;
mod path;

use parser::{Lang, CST};

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    source_path: std::path::PathBuf,
    #[structopt(parse(from_os_str))]
    target_path: std::path::PathBuf,
}

fn main() -> Result<()> {
    let Cli {
        source_path,
        target_path,
    } = Cli::from_args();

    let mut target_file = target_path;

    if target_file.is_dir() {
        let file_name = source_path.file_name().unwrap();
        target_file.push(file_name);
    }

    let canonicalized_source_path = fs::canonicalize(&source_path)?;
    let affected_files = grep::find_affected_files(&canonicalized_source_path)?;

    fs::rename(&source_path, &target_file)?;
    let canonicalized_target_path = fs::canonicalize(&target_file)?;

    let source_code = fs::read_to_string(&target_file)?;
    let new_source_code = update_imports(source_code, &source_path, &target_file)?;

    fs::write(target_file, new_source_code)?;

    for affected_file in affected_files.iter() {
        let affected_file = fs::canonicalize(affected_file)
            .map_err(|_| anyhow!("can't find {:?}", affected_file))?;

        let affected_source_code = fs::read_to_string(&affected_file)?;

        let updated_source_code = update_import(
            &affected_source_code,
            &affected_file,
            &canonicalized_source_path,
            &canonicalized_target_path,
        )?;

        fs::write(affected_file, updated_source_code)?;
    }

    Ok(())
}

fn infer_langauge_from_suffix(file_name: &PathBuf) -> Result<Lang> {
    let suffix = file_name
        .extension()
        .and_then(|os_str| os_str.to_str())
        .ok_or(anyhow!("Missing suffix on file"))?;

    match suffix {
        "ts" => Ok(Lang::TypeScript),
        "tsx" => Ok(Lang::TypeScriptTsx),
        suffix => Err(anyhow!("{:?} files are not supported", suffix)),
    }
}

fn update_imports(
    source_code: String,
    source_file: &PathBuf,
    target_file: &PathBuf,
) -> Result<String> {
    let source_dir = path::get_parent(&source_file);
    let target_dir = path::get_parent(&target_file);

    if source_dir.eq(&target_dir) {
        return Ok(source_code);
    }

    let lang = infer_langauge_from_suffix(&source_file)?;
    let mut concrete_syntax_tree = CST::new(&source_code, lang)?;

    concrete_syntax_tree.replace_all_imports(|import_string| {
        let path = import_string::to_path(&source_file, &import_string)?;
        let new_import = import_string::from_paths(&target_file, &path);
        new_import
    })?;

    Ok(concrete_syntax_tree.get_source_code())
}

fn update_import(
    source_code: &String,
    source_file: &PathBuf,
    old_import_location: &PathBuf,
    new_import_location: &PathBuf,
) -> Result<String> {
    let lang = infer_langauge_from_suffix(&source_file)?;
    let mut concrete_syntax_tree = CST::new(&source_code, lang)?;

    let old_import = import_string::from_paths(source_file, old_import_location)?;
    let new_import = import_string::from_paths(source_file, new_import_location)?;

    concrete_syntax_tree.replace_one_import(&old_import, &new_import)?;

    Ok(concrete_syntax_tree.get_source_code())
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::path::PathBuf;

    #[test]
    fn it_updates_imports_0() -> Result<()> {
        let code: String = r#"
            import some from '../../some';
            import other from '../../other';
            function main() {
                console.log("hullo world");
            }
            "#
        .into();

        let source: PathBuf = "/src/a/b/c/d/source.ts".into();
        let target: PathBuf = "/src/a/b/c/d/e/target.ts".into();

        let new_source_code = super::update_imports(code, &source, &target)?;

        let new_import_0: String = "import some from '../../../some';".into();
        let new_import_1: String = "import other from '../../../other';".into();

        assert!(new_source_code.contains(&new_import_0));
        assert!(new_source_code.contains(&new_import_1));
        Ok(())
    }

    #[test]
    fn it_updates_imports_1() -> Result<()> {
        let code: String = r#"
            import some from '../../some';
            import other from '../../other';
            function main() {
                console.log("hullo world");
            }
            "#
        .into();

        let source: PathBuf = "/src/a/b/c/d/source.ts".into();
        let target: PathBuf = "/src/a/target.ts".into();

        let new_source_code = super::update_imports(code, &source, &target)?;

        let new_import_0: String = "import some from './b/some';".into();
        let new_import_1: String = "import other from './b/other';".into();

        assert!(new_source_code.contains(&new_import_0));
        assert!(new_source_code.contains(&new_import_1));
        Ok(())
    }
}
