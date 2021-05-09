use anyhow::{anyhow, Result};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
// use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::thread;
// use std::time::Instant;
use structopt::StructOpt;

mod edit;
mod grep;
mod import_string;
mod parser;
mod path;

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

    let current_dir = env::current_dir()?;

    if source_path.is_dir() {
        rename_dir(current_dir, source_path, target_path)
    } else {
        rename_single_file(current_dir, source_path, target_path)
    }
}

fn rename_single_file(
    current_dir: PathBuf,
    source_path: PathBuf,
    target_path: PathBuf,
) -> Result<()> {
    let mut target_file = target_path;

    if target_file.is_dir() {
        let file_name = source_path.file_name().unwrap();
        target_file.push(file_name);
    }

    let full_source_path = path::join(&current_dir, &source_path)?;
    let full_target_path = path::join(&current_dir, &target_file)?;

    let source = source_path.clone();
    let handler = thread::spawn(move || match move_file(&source, &target_file) {
        Ok(_) => (),
        Err(err) => println!("{:?}", err),
    });

    let other_files: Vec<PathBuf> = grep::iter_files(&current_dir)
        .filter(|path| !path.eq(&full_target_path) && !path.eq(&full_source_path))
        .collect();

    other_files
        .into_par_iter()
        .try_for_each(move |affected_file| {
            let affected_file = path::join(&current_dir, &affected_file)?;

            let source_code = fs::read_to_string(&affected_file)
                .map_err(|_| anyhow!("Could not find {:?}", affected_file))?;

            let import_string = import_string::from_paths(&affected_file, &full_source_path)?;
            let import_string = import_string::to_node_import(&import_string);

            let contains_import = source_code.contains(&import_string);

            if !contains_import {
                return Ok(());
            }

            let updated_source_code = edit::move_required_file(
                &source_code,
                &affected_file,
                &full_source_path,
                &full_target_path,
            )?;

            fs::write(&affected_file, updated_source_code)
                .map_err(|_| anyhow!("Failed to write {:?}", affected_file))
        })?;

    handler.join().unwrap();
    Ok(())
}

fn move_file(source_path: &PathBuf, target_file: &PathBuf) -> Result<()> {
    fs::rename(&source_path, &target_file)?;
    let source_code = fs::read_to_string(&target_file)?;
    let new_source_code = edit::move_source_file(source_code, &source_path, &target_file)?;
    fs::write(target_file, new_source_code)?;
    Ok(())
}

fn rename_dir(current_dir: PathBuf, source_path: PathBuf, target_path: PathBuf) -> Result<()> {
    let full_source_path = path::join(&current_dir, &source_path)?;
    let full_target_path = path::join(&current_dir, &target_path)?;

    let moved_files: Result<Vec<(PathBuf, PathBuf)>> = grep::iter_files(&full_source_path)
        .map(|file| {
            let rel_path = path::diff(&full_source_path, &file)?;
            let new_file = path::join(&&full_target_path, &rel_path)?;
            Ok((file, new_file))
        })
        .collect();

    let moved_files = moved_files?;
    let moved_files = &moved_files;

    moved_files
        .into_par_iter()
        .try_for_each(|(source_file, target_file)| -> Result<()> {
            let source_code = fs::read_to_string(&source_file)
                .map_err(|_| anyhow!("Failed to read {:?}", source_file));
            let source_code = source_code?;

            let new_source_code =
                edit::replace_imports(&source_file, &source_code, |import_string| {
                    let has_moved = moved_files.into_iter().find(|(moved_file, _)| {
                        import_string::is_import_from(&source_file, moved_file, &import_string)
                            .unwrap_or(false)
                    });

                    match has_moved {
                        Some((old_location, new_location)) => {
                            let args = import_string::RequiredFileRename {
                                source_file: &source_file,
                                import_string,
                                old_location: &old_location,
                                new_location: &new_location,
                            };
                            let import_string = import_string::rename_required_file(&args)?;
                            let args = import_string::SourceFileRename {
                                import_string: &&import_string,
                                old_location: &source_file,
                                new_location: &target_file,
                            };
                            import_string::rename_source_file(&args)
                        }
                        None => {
                            let args = import_string::SourceFileRename {
                                import_string: &&import_string,
                                old_location: &source_file,
                                new_location: &target_file,
                            };
                            import_string::rename_source_file(&args)
                        }
                    }
                })?;
            fs::write(source_file, new_source_code)
                .map_err(|_| anyhow!("Failed to write {:?}", source_file))?;
            Ok(())
        })?;
    fs::rename(&source_path, &target_path)?;

    // let other_files: Vec<PathBuf> = grep::iter_files(&current_dir)
    //     .filter(|file| {
    //         moved_files
    //             .clone()
    //             .into_iter()
    //             .find(|(moved_file, _)| file.eq(moved_file))
    //             .is_none()
    //     })
    //     .collect();

    // other_files
    //     .into_par_iter()
    //     .try_for_each(move |affected_file| {
    //         let affected_file = path::join(&current_dir, &affected_file)?;

    //         let source_code = fs::read_to_string(&affected_file)
    //             .map_err(|_| anyhow!("Could not find {:?}", affected_file))?;

    //         let import_string = import_string::from_paths(&affected_file, &full_source_path)?;
    //         let import_string = import_string::to_node_import(&import_string);

    //         let contains_import = source_code.contains(&import_string);

    //         if !contains_import {
    //             return Ok(());
    //         }

    //         let updated_source_code = edit::move_required_file(
    //             &source_code,
    //             &affected_file,
    //             &full_source_path,
    //             &full_target_path,
    //         )?;

    //         fs::write(&affected_file, updated_source_code)
    //             .map_err(|_| anyhow!("Failed to write {:?}", affected_file))
    //     })?;

    Ok(())
}
