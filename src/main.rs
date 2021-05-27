use anyhow::{anyhow, Result};
use clap::{crate_authors, crate_description, crate_version, App, Arg};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use ropey::Rope;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::thread;

mod edit;
mod grep;
mod import_string;
mod import_updater;
mod parser;
mod path;
mod tsconfig;

const SRC_ARG: &str = "source";
const TARGET_ARG: &str = "target";

fn main() -> Result<()> {
    let matches = App::new("mvts")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::new(SRC_ARG)
                .about("source path")
                .index(1)
                .multiple(true)
                .required(true),
        )
        .arg(
            Arg::new(TARGET_ARG)
                .about("target path")
                .index(2)
                .required(true),
        )
        .get_matches();

    let current_dir = std::env::current_dir()?;

    let src_path_args: Result<Vec<PathBuf>> = matches
        .values_of(SRC_ARG)
        .unwrap()
        .map(|path_str| path_str.into())
        .map(|rel_path: PathBuf| {
            fs::canonicalize(rel_path.clone())
                .map_err(|_| anyhow!("Can't find file {:?}", &rel_path))
        })
        .collect();
    let src_path_args = src_path_args?;

    // FIX: Handle errors here!
    let target_path_arg: PathBuf = matches.value_of(TARGET_ARG).unwrap().into();
    let target_path_arg = current_dir.join(target_path_arg);

    if target_path_arg.is_file() {
        panic!("I Don't want to overwrite existing file.");
    }

    if src_path_args.len() > 1 {
        if !target_path_arg.exists() {
            fs::create_dir_all(target_path_arg.clone())?;
        }
    }

    let tsconfig_file = tsconfig::find_ts_config(&current_dir);

    if tsconfig_file.is_none() {
        println!("Could not find tsconfig.json");
    }

    let path_map = tsconfig_file
        .clone()
        .and_then(|tsconfig_file| tsconfig::get_path_map_from_ts_config(&tsconfig_file));

    // NB: Hard coded to src, this can be extracted from tsconfig_file
    let project_root = tsconfig_file
        .and_then(|tsconfig_file| {
            tsconfig_file
                .parent()
                .map(|root_path| root_path.to_path_buf())
            // .map(|root_path| root_path.join("src"))
        })
        .unwrap_or(current_dir);

    println!("{:?}", target_path_arg);
    let target_path_arg = path::diff(&project_root, &target_path_arg)?;

    println!("{:?}", src_path_args);

    let src_path_args: Result<Vec<PathBuf>> = src_path_args
        .into_iter()
        .map(|source_path| {
            println!("{:?}", source_path);
            path::diff(&project_root, &source_path)
        })
        .collect();

    let src_path_args = src_path_args?;

    // TODO FIX:
    // project files are full paths
    // args are partial paths,
    // make them eq.
    // IMO best make them all to be files from src/

    let project_files: Result<HashMap<PathBuf, (PathBuf, String)>> =
        grep::iter_files(&project_root)
            .map(|file_path| {
                let source_code = fs::read_to_string(&file_path)?;

                let file_path = path::diff(&project_root, &file_path)?;

                let moved_file_path = src_path_args
                    .iter()
                    .find_map(|moved_path| {
                        path::move_path(&file_path, &moved_path, &target_path_arg)
                    })
                    .unwrap_or_else(|| file_path.clone());

                Ok((file_path, (moved_file_path, source_code)))
            })
            .collect();

    let project_files = project_files?;

    for (a, (b, _)) in project_files.iter() {
        println!("{:?}, {:?}", a, b);
    }

    println!("{}", project_files.len());

    // let lol = matches.source_files;

    // let current_dir = env::current_dir()?;
    // let full_source_path = path::join(&current_dir, &source_path)?;

    // let v: Result<Vec<(PathBuf, String)>> = grep::iter_files(&full_source_path)
    //     .map(|file_path| {
    //         let source_code = fs::read_to_string(&file_path)?;
    //         Ok((file_path, source_code))
    //     })
    //     .collect();

    // let paths = tsconfig::read_ts_config(&current_dir);
    // println!("{:?}", paths);

    // println!("{:?}", ts_path);

    // let edited = v
    //     .into_par_iter()
    //     .filter_map(|(file_path, source_code)| {
    //         match path::move_path(&file_path, &full_source_path, &full_target_path) {
    //             Some(new_path) => (),
    //             None => (),
    //         }
    //         // parser::replace_imports(&file_path, &source_code, |str| Ok(str.clone())).unwrap_or(None)
    //     })
    //     .collect();

    // if source_path.is_dir() {
    //     rename_dir(current_dir, source_path, target_path)
    // } else {
    //     rename_single_file(current_dir, source_path, target_path)
    // }
    Ok(())
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
        .try_for_each(move |affected_file| -> Result<()> {
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

            if !source_code.eq(&updated_source_code) {
                fs::write(&affected_file, updated_source_code)
                    .map_err(|_| anyhow!("Failed to write {:?}", affected_file))?
            }

            Ok(())
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
            let new_file = path::join(&full_target_path, &rel_path)?;
            Ok((file, new_file))
        })
        .collect();

    let moved_files = &moved_files?;

    moved_files
        .into_par_iter()
        .try_for_each(|(source_file, target_file)| -> Result<()> {
            let source_code = fs::read_to_string(&source_file)
                .map_err(|_| anyhow!("Failed to read {:?}", source_file))?;

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

            if !source_code.eq(&new_source_code) {
                fs::write(source_file, new_source_code)
                    .map_err(|_| anyhow!("Failed to write {:?}", source_file))?;
            }

            Ok(())
        })?;

    let other_files: Vec<PathBuf> = grep::iter_files(&current_dir)
        .filter(|path| {
            moved_files
                .into_iter()
                .find(|(moved_path, _)| moved_path.eq(path))
                .is_none()
        })
        .collect();

    other_files
        .into_par_iter()
        .try_for_each(|source_file| -> Result<()> {
            let source_code = fs::read_to_string(&source_file)
                .map_err(|_| anyhow!("Failed to read {:?}", source_file))?;

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
                            import_string::rename_required_file(&args)
                        }
                        None => Ok(import_string.clone()),
                    }
                })?;

            if !source_code.eq(&new_source_code) {
                fs::write(&source_file, new_source_code)
                    .map_err(|_| anyhow!("Failed to write {:?}", source_file))?;
            }

            Ok(())
        })?;

    fs::rename(&source_path, &target_path)
        .map_err(|_| anyhow!("Failed to rename {:?} to {:?}", &source_path, &target_path,))?;

    Ok(())
}
