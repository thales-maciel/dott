use crate::prelude::*;

mod error;
mod prelude;

use git2::Repository;
use itertools::Itertools;
use glob::{glob, Pattern};
use std::{fs::{create_dir_all, read_to_string, copy}, env, path::PathBuf};
use directories::{BaseDirs, ProjectDirs};
use clap::{Args, Parser, Subcommand};

pub fn assert_repo_exists(dir: &PathBuf) -> Result<()> {
    match Repository::init(dir) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::Generic("Failed to initialize repository".into())),
    }
}

pub fn init() -> Result<()> {
    let data_dir = get_data_dir();
    if let Err(_) = create_dir_all(&data_dir) {
        return Err(Error::Generic("Failed to create data directory".into()));
    };
    return assert_repo_exists(&data_dir);
}

pub fn get_ignored_globs() -> Option<Vec<String>> {
    if let Ok(lines) = read_to_string(get_ignore_file()) {
        return Some(lines.lines().map(|l| l.to_string()).collect());
    };
    None
}

pub fn get_config_globs() -> Option<Vec<String>> {
    if let Ok(lines) = read_to_string(get_config_file()) {
        return Some(lines.lines().map(|l| l.to_string()).collect());
    };
    None
}

pub fn get_ignore_file() -> PathBuf {
    get_data_dir().join(".gitignore").into()
}

pub fn get_config_file() -> PathBuf {
    ProjectDirs::from("dev", "thales-maciel", "dotr").unwrap().config_dir().join("dotr.config").to_str().unwrap().into()
}

pub fn get_home_dir() -> PathBuf {
    BaseDirs::new().unwrap().home_dir().to_owned()
}

pub fn get_data_dir() -> PathBuf {
    ProjectDirs::from("dev", "thales-maciel", "dotr").unwrap().data_dir().to_owned()
}

pub fn get_absolute_path(path: &PathBuf) -> PathBuf {
    PathBuf::from(get_home_dir()).join(path)
}

pub fn get_files_to_sync() -> Result<Vec<PathBuf>> {
    if let Some(globs) = get_config_globs() {
        let res: std::result::Result<Vec<PathBuf>, glob::GlobError> = globs
            .into_iter()
            .flat_map(|pattern| {
                glob(&pattern).expect("Failed to read glob pattern")
            })
            // .map(|entry| {
            //     entry.map(|path| get_absolute_path(&path))
            // })
            .collect();

        if let Ok(paths) = res {
            return Ok(paths.into_iter().unique().collect());
        }
        return Err(Error::Generic("Failed to read glob pattern".into()));
    };
    Ok(vec![])
}

pub fn sync() -> Result<()> {
    if let Ok(paths) = get_files_to_sync() {
        let ignore_globs = get_ignored_globs();
        if let Some(ignore_globs) = ignore_globs {
            // create vector of Patterns from ignore globs
            let ignore_patterns: Vec<Pattern> = ignore_globs.into_iter().map(|g| Pattern::new(&g).unwrap()).collect();
            // for path in paths
            for path in paths {
                // if path is not ignored
                if !ignore_patterns.iter().any(|p| p.matches(&path.to_str().unwrap())) {
                    // copy path to data_dir
                    if let Err(e) = copy(&path, get_data_dir()) {
                        println!("path {:#?}", &path);
                        println!("err {:#?}", e);
                        return Err(Error::Generic("Failed to copy file".into()));
                    }
                }
            }
        } else {
            for path in paths {
                // ensure dir exists before copying
                if let Err(e) = create_dir_all(get_data_dir().join(path.parent().unwrap())) {
                    println!("path {:#?}", &path);
                    println!("err {:#?}", e);
                    return Err(Error::Generic("Failed to create parent dir".into()));
                }
                if let Err(e) = copy(&path, get_data_dir().join(path.to_str().unwrap())) {
                    println!("path {:#?}", &path);
                    println!("secarg {:#?}", get_data_dir().join(path.to_str().unwrap()));
                    println!("err out {:#?}", e);
                    return Err(Error::Generic("Failed to copy file".into()));
                }
                println!("data_dir is {:#?}", get_data_dir());
                println!("Copying {:#?} to {:#?}", &path, get_data_dir().join(path.to_str().unwrap()));
            }
        }
        return Ok(());
    }
    Err(Error::Generic("Failed to copy files".into()))
}

fn main() -> Result<()> {
    // Always run from home dir
    let home = get_home_dir();
    env::set_current_dir(home)?;

    let cli = Cli::parse();
    match &cli.command {
        Commands::Init => {
            init()?;
        },
        Commands::Sync => {
            sync()?;
        },
        Commands::Pwd => {
            println!("{}", get_data_dir().display());
        }
        _ => {}
    };
    Ok(())
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Adds paths to track
    Add(AddArgs),
    /// Initializes the Dotr repository
    Init,
    /// Updates the Dotr repository with all tracked files
    Sync,
    /// Prints the Dotr repository directory location
    Pwd,
    /// Lists all tracked files
    Ls,
    /// Places all tracked files into their destination
    Install(InstallArgs)
}

#[derive(Args)]
pub struct InstallArgs {
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Args)]
pub struct AddArgs {
    pub paths: Vec<String>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 3);
    }
}
