use std::{
    path::PathBuf,
    fs::{read_to_string, copy, remove_file, create_dir_all},
    env,
};
use console::{style, Term};
use dialoguer::Confirm;
use glob::glob;
use path_absolutize::Absolutize;

use crate::prelude::*;

pub mod error;
pub mod prelude;

pub struct Add { from: PathBuf, to: PathBuf}
pub struct Overwrite { from: PathBuf, to: PathBuf}
pub struct Remove(PathBuf);

pub fn sync_dirs(pattern_file: &PathBuf, from_dir: &PathBuf, to_dir: &PathBuf, raw: &bool) -> Result<()> {
    // resolve absolute paths
    let from_dir = from_dir.absolutize().map_err(|_| DotrError::PathNotFound(f!("{:?}", from_dir)))?;
    let to_dir = to_dir.absolutize().map_err(|_| DotrError::PathNotFound(f!("{:?}", to_dir)))?;

    // assert from_dir and to_dir are directories
    if !from_dir.exists() {
        return Err(DotrError::PathNotFound(f!("{:?}", from_dir)));
    }
    if !from_dir.is_dir() {
        return Err(DotrError::NotDir(f!("{:?}", from_dir)));
    }
    if !to_dir.exists() {
        return Err(DotrError::PathNotFound(f!("{:?}", to_dir)));
    }
    if !to_dir.is_dir() {
        return Err(DotrError::NotDir(f!("{:?}", to_dir)));
    }
    // assert pattern_file is a file
    if !pattern_file.exists() {
        return Err(DotrError::PathNotFound(f!("{:?}", pattern_file)));
    }
    if !pattern_file.is_file() {
        return Err(DotrError::NotFile(f!("{:?}", pattern_file)));
    }

    // go to from_dir
    env::set_current_dir(&from_dir).map_err(DotrError::IO)?;

    // get all patterns from file
    let patterns = read_to_string(pattern_file)
        .map_err(DotrError::IO)?
        .lines()
        .map(|l| l.to_string())
        .collect::<Vec<String>>();

    // get all matching files
    let mut add_ops: Vec<Add> = Vec::new();
    let mut overwrite_ops: Vec<Overwrite> = Vec::new();
    let mut remove_ops: Vec<Remove> = Vec::new();

    let mut files: Vec<PathBuf> = Vec::new();
    for pattern in &patterns {
        let paths = glob(&pattern).map_err(|e| DotrError::BadGlob(pattern.clone(), e))?;
        for path in paths {
            let path = path.map_err(|e| DotrError::PathAccess(pattern.clone(), e))?;
            if path.is_dir() { continue }
            let absolute_path = from_dir.join(&path);
            if files.iter().find(|f| f.to_owned() == &absolute_path).is_none() {
                files.push(absolute_path.clone());
                let target_path = to_dir.join(&path);
                if target_path.exists() {
                    overwrite_ops.push(Overwrite { from: absolute_path.clone(), to: target_path })
                } else {
                    add_ops.push(Add { from: absolute_path, to: target_path })
                }
            }
        }
    }

    // go to to_dir
    env::set_current_dir(&to_dir).map_err(DotrError::IO)?;

    // get all matching destination files
    let mut files_to_delete: Vec<PathBuf> = Vec::new();
    for pattern in &patterns {
        let paths = glob(&pattern).map_err(|e| DotrError::BadGlob(pattern.clone(), e))?;
        for path in paths {
            let path = path.map_err(|e| DotrError::PathAccess(pattern.clone(), e))?;
            if path.is_dir() { continue }
            let absolute_path = to_dir.join(path);
            // find out if path is going to be overwritten
            if overwrite_ops.iter().find(|o| o.to == absolute_path).is_some() {
                continue
            }
            if files_to_delete.iter().find(|f| f.to_owned() == &absolute_path).is_none() {
                files_to_delete.push(absolute_path.clone());
                remove_ops.push(Remove(absolute_path));
            }
        }
    }

    if add_ops.is_empty() && overwrite_ops.is_empty() && remove_ops.is_empty() {
        println!("No syncing necessary");
        return Ok(());
    }

    if !add_ops.is_empty() {
        println!("The following files will be added to {}", to_dir.display());
        for add in add_ops.iter() {
            println!("{}", style(add.to.display()).green());
        }
        println!();
    }

    if !overwrite_ops.is_empty() {
        println!("The following files will be overwritten in {}", to_dir.display());
        for overwrite in overwrite_ops.iter() {
            println!("{}", style(overwrite.to.display()).yellow());
        }
        println!();
    }

    if !remove_ops.is_empty() {
        println!("The following files will be removed from {}", to_dir.display());
        for remove in remove_ops.iter() {
            println!("{}", style(remove.0.display()).red());
        }
        println!();
    }

    if raw.to_owned() {
        return Ok(())
    }

    // ask the user to confirm
    if Confirm::new()
        .wait_for_newline(true)
        .default(true)
        .show_default(true)
        .with_prompt("Do you want to continue?")
        .interact_on(&Term::stdout())? {
        // add all files
        for add in add_ops.iter() {
            create_dir_all(&add.to.parent().unwrap())?;
            copy(&add.from, &add.to)?;
        }
        // overwrite all files
        for overwrite in overwrite_ops.iter() {
            create_dir_all(&overwrite.to.parent().unwrap())?;
            copy(&overwrite.from, &overwrite.to)?;
        }
        // remove all files
        for remove in remove_ops.iter() {
            remove_file(&remove.0)?;
        }
    } 

    Ok(())
}

