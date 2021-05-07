use anyhow::{anyhow, Result};
use std::env;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

mod edit;
mod grep;
mod import_string;
mod parser;
mod path;

use edit::{update_import, update_imports};
use path::normalize;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    source_path: PathBuf,
    #[structopt(parse(from_os_str))]
    target_path: PathBuf,
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

    let current_dir = env::current_dir()?;

    let affected_files = grep::find_affected_files(&current_dir, &source_path)?;

    move_file(&source_path, &target_file)?;

    let full_source_path = get_full_path(&current_dir, &source_path)?;
    let full_target_path = get_full_path(&current_dir, &target_file)?;

    for affected_file in affected_files.iter() {
        let affected_file = get_full_path(&current_dir, affected_file)?;

        let affected_source_code = fs::read_to_string(&affected_file)
            .map_err(|_| anyhow!("Could not find {:?}", affected_file))?;

        let updated_source_code = update_import(
            &affected_source_code,
            &affected_file,
            &full_source_path,
            &full_target_path,
        )?;

        fs::write(affected_file, updated_source_code)?;
    }

    Ok(())
}

fn get_full_path(current_dir: &PathBuf, path: &PathBuf) -> Result<PathBuf> {
    let full_path = current_dir.join(path);
    normalize(&full_path)
}

fn move_file(source_path: &PathBuf, target_file: &PathBuf) -> Result<()> {
    fs::rename(&source_path, &target_file)?;
    let source_code = fs::read_to_string(&target_file)?;
    let new_source_code = update_imports(source_code, &source_path, &target_file)?;
    fs::write(target_file, new_source_code)?;
    Ok(())
}
