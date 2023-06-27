use console::{style, Term};
use dialoguer::Confirm;
use glob::glob;
use path_absolutize::Absolutize;
use std::{
    env,
    fs::{copy, create_dir_all, read_to_string, remove_file},
    path::PathBuf,
};

use crate::prelude::*;

pub mod error;
pub mod prelude;

pub struct Add {
    from: PathBuf,
    to: PathBuf,
}
pub struct Overwrite {
    from: PathBuf,
    to: PathBuf,
}
pub struct Remove(PathBuf);

fn validate_paths(pattern_file: &PathBuf, from_dir: &PathBuf, to_dir: &PathBuf) -> Result<(PathBuf, PathBuf)> {
    // resolve absolute paths
    let from_dir = from_dir
        .absolutize()
        .map_err(|_| DottError::PathNotFound(f!("{:?}", from_dir)))?.to_path_buf();
    let to_dir = to_dir
        .absolutize()
        .map_err(|_| DottError::PathNotFound(f!("{:?}", to_dir)))?.to_path_buf();

    // assert from_dir and to_dir are directories
    if !from_dir.exists() {
        return Err(DottError::PathNotFound(f!("{:?}", from_dir)));
    }
    if !from_dir.is_dir() {
        return Err(DottError::NotDir(f!("{:?}", from_dir)));
    }
    if !to_dir.exists() {
        return Err(DottError::PathNotFound(f!("{:?}", to_dir)));
    }
    if !to_dir.is_dir() {
        return Err(DottError::NotDir(f!("{:?}", to_dir)));
    }
    // assert pattern_file is a file
    if !pattern_file.exists() {
        return Err(DottError::PathNotFound(f!("{:?}", pattern_file)));
    }
    if !pattern_file.is_file() {
        return Err(DottError::NotFile(f!("{:?}", pattern_file)));
    }
    return Ok((from_dir, to_dir))
}

pub fn sync_dirs(
    pattern_file: &PathBuf,
    from_dir: &PathBuf,
    to_dir: &PathBuf,
    raw: &bool,
    skip_prompt: &bool
) -> Result<()> {
    let (from_dir, to_dir) = validate_paths(pattern_file, from_dir, to_dir)?;

    let Some((add_ops, overwrite_ops, remove_ops)) = compute_operations(&from_dir, pattern_file, &to_dir)? else {
        return Ok(());
    };

    print_operations(&add_ops, &to_dir, &overwrite_ops, &remove_ops);

    if raw.to_owned() {
        return Ok(());
    }

    if skip_prompt.to_owned() {
        perform_operations(add_ops, overwrite_ops, remove_ops)?;
    } else {
        // ask the user to confirm
        if Confirm::new()
            .wait_for_newline(true)
            .default(true)
            .show_default(true)
            .with_prompt("Do you want to continue?")
            .interact_on(&Term::stdout())?
        {
            perform_operations(add_ops, overwrite_ops, remove_ops)?;
        };
    };

    Ok(())
}

fn perform_operations(add_ops: Vec<Add>, overwrite_ops: Vec<Overwrite>, remove_ops: Vec<Remove>) -> Result<()> {
    for remove in remove_ops.iter() {
        remove_file(&remove.0)?;
    }
    for add in add_ops.iter() {
        create_dir_all(add.to.parent().unwrap())?;
        copy(&add.from, &add.to)?;
    }
    for overwrite in overwrite_ops.iter() {
        create_dir_all(overwrite.to.parent().unwrap())?;
        copy(&overwrite.from, &overwrite.to)?;
    }
    println!("{}", style("Sync completed successfully").bold());
    Ok(())
}

fn print_operations(add_ops: &Vec<Add>, to_dir: &PathBuf, overwrite_ops: &Vec<Overwrite>, remove_ops: &Vec<Remove>) {
    if !add_ops.is_empty() {
        println!("The following files will be added to {}", to_dir.display());
        for add in add_ops.iter() {
            println!("{}", style(add.to.display()).green());
        }
        println!();
    }

    if !overwrite_ops.is_empty() {
        println!(
            "The following files will be overwritten in {}",
            to_dir.display()
        );
        for overwrite in overwrite_ops.iter() {
            println!("{}", style(overwrite.to.display()).yellow());
        }
        println!();
    }

    if !remove_ops.is_empty() {
        println!(
            "The following files will be removed from {}",
            to_dir.display()
        );
        for remove in remove_ops.iter() {
            println!("{}", style(remove.0.display()).red());
        }
        println!();
    }
}

fn compute_operations(
    from_dir: &PathBuf,
    pattern_file: &PathBuf,
    to_dir: &PathBuf
) -> Result<Option<(Vec<Add>, Vec<Overwrite>, Vec<Remove>)>> {
    env::set_current_dir(&from_dir).map_err(DottError::IO)?;
    let patterns = read_to_string(pattern_file)
        .map_err(DottError::IO)?
        .lines()
        .map(|l| l.to_string())
        .collect::<Vec<String>>();
    let mut add_ops: Vec<Add> = Vec::new();
    let mut overwrite_ops: Vec<Overwrite> = Vec::new();
    let mut remove_ops: Vec<Remove> = Vec::new();
    let mut files: Vec<PathBuf> = Vec::new();
    for pattern in &patterns {
        let paths = glob(pattern).map_err(|e| DottError::BadGlob(pattern.clone(), e))?;
        for path in paths {
            let path = path.map_err(|e| DottError::PathAccess(pattern.clone(), e))?;
            if path.is_dir() {
                continue;
            }
            let absolute_path = from_dir.join(&path);
            if !files.iter().any(|f| f == &absolute_path) {
                files.push(absolute_path.clone());
                let target_path = to_dir.join(&path);
                if target_path.exists() {
                    overwrite_ops.push(Overwrite {
                        from: absolute_path.clone(),
                        to: target_path,
                    })
                } else {
                    add_ops.push(Add {
                        from: absolute_path,
                        to: target_path,
                    })
                }
            }
        }
    }
    env::set_current_dir(to_dir).map_err(DottError::IO)?;
    let mut files_to_delete: Vec<PathBuf> = Vec::new();
    for pattern in &patterns {
        let paths = glob(pattern).map_err(|e| DottError::BadGlob(pattern.clone(), e))?;
        for path in paths {
            let path = path.map_err(|e| DottError::PathAccess(pattern.clone(), e))?;
            if path.is_dir() {
                continue;
            }
            let absolute_path = to_dir.join(path);
            // find out if path is going to be overwritten
            if overwrite_ops.iter().any(|o| o.to == absolute_path) {
                continue;
            }
            if !files_to_delete.iter().any(|f| f == &absolute_path) {
                files_to_delete.push(absolute_path.clone());
                remove_ops.push(Remove(absolute_path));
            }
        }
    }
    if add_ops.is_empty() && overwrite_ops.is_empty() && remove_ops.is_empty() {
        return Ok(None)
    };
    Ok(Some((add_ops, overwrite_ops, remove_ops)))
}
