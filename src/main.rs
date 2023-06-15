#![allow(unused)]

use crate::prelude::*;

mod error;
mod prelude;

use clap::{Args, Parser, Subcommand};
use directories::{BaseDirs, ProjectDirs};
use git2::Repository;
use glob::{glob, Pattern};
use itertools::Itertools;
use std::{
    env,
    fs::{copy, remove_file, create_dir_all, read_to_string},
    path::PathBuf,
};


pub trait SyncedDir {
    fn get_file_paths(&self) -> Result<Vec<PathBuf>>;
    fn update_files(&self, source: &Slice) -> Result<()>;
}


pub struct Slice {
    root_path: PathBuf,
    globs: Vec<String>,
    ignore_patterns: Vec<String>,
}

fn is_ignored(ignore_patterns: &Vec<String>, path: &PathBuf, root_path: &PathBuf) -> bool {
    ignore_patterns
        .into_iter()
        .map(|g| Pattern::new(root_path.join(g).to_str().unwrap()).unwrap())
        .any(|p| p.matches(&path.to_str().unwrap()))
}

impl SyncedDir for Slice {
    fn get_file_paths(&self) -> Result<Vec<PathBuf>> {
        self.globs.clone().into_iter()
            .flat_map(|pattern| glob(&pattern).unwrap())
            .map(|v| v.map_err(|_| Error::Generic("Failed to read glob pattern".into())))
            .collect::<Result<Vec<_>>>()
            .map(|paths| paths.clone().into_iter()
                .unique()
                .map(|p| self.root_path.join(p))
                .filter(|path| path.is_file() && !is_ignored(&self.ignore_patterns, path, &self.root_path))
                .collect())
            .map_err(|_| Error::Generic("Failed to read glob pattern".into()))
    }

    fn update_files(&self, source: &Slice) -> Result<()> {
        let current_files = self.get_file_paths()?;
        for file in current_files {
            println!("Removing file {:?}", file);
            remove_file(file)?;
        }
        let files_to_copy = source.get_file_paths()?;
        for file in files_to_copy {
            let relative_path = &file.strip_prefix(&source.root_path).unwrap();
            let absolute_path = self.root_path.join(relative_path);
            if let Err(_) = create_dir_all(absolute_path.parent().unwrap()) {
                return Err(Error::Generic("Failed to create parent dir".into()));
            }
            println!("Copying {:#?} to {:#?}", file, absolute_path);
            if let Err(_) = copy(file, absolute_path) {
                return Err(Error::Generic("Failed to copy file".into()));
            }
        }
        Ok(())
    }
}

impl Slice {
    fn new(root_path: PathBuf, globs: Vec<String>, ignore_patterns: Vec<String>) -> Self {
        Self {root_path, globs, ignore_patterns}
    }
}

pub struct Dotr {
    repo: Slice,
    home: Slice
}

impl Dotr {
    fn new(repo: Slice, home: Slice) -> Result<Self> {
        Ok(Self {repo, home})
    }

    fn sync(&self) -> Result<()> {
        self.repo.update_files(&self.home)
    }

    fn install(&self) -> Result<()> {
        self.home.update_files(&self.repo)
    }
}

pub fn assert_repo_exists(dir: &PathBuf) -> Result<()> {
    if let Err(_) = create_dir_all(&dir) {
        return Err(Error::Generic("Failed to create data directory".into()));
    };

    match Repository::init(dir) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::Generic("Failed to initialize repository".into())),
    }
}

pub fn get_patterns_from_file(file_path: &PathBuf) -> Vec<String> {
    if let Ok(lines) = read_to_string(file_path) {
        return lines.lines().map(|l| l.to_string()).collect();
    };
    vec![]
}

pub fn get_repo_dir() -> Result<PathBuf> {
    let Some(project_dir) = ProjectDirs::from("dev", "thales-maciel", "dotr") else {
        return Err(Error::Generic("No valid path could be retrieved from system".into()))
    };
    Ok(project_dir.data_dir().to_owned())
}

pub fn get_ignore_file() -> PathBuf {
    get_repo_dir().unwrap().join(".gitignore").into()
}

// todo: make this a get_or_create instead
pub fn get_config_file() -> Result<PathBuf> {
    let Some(project_dir) = ProjectDirs::from("dev", "thales-maciel", "dotr") else {
        return Err(Error::Generic("No valid path could be retrieved from system".into()))
    };
    Ok(project_dir.config_dir().join("dotr.config"))
}

pub fn get_home_dir() -> Result<PathBuf> {
    let Some(base_dir) = BaseDirs::new() else {
        return Err(Error::Generic("No valid path could be retrieved from system".into()))
    };
    Ok(base_dir.home_dir().to_owned())
}

fn main() -> Result<()> {
    let home_dir = get_home_dir()?;

    // Always run from home dir
    env::set_current_dir(&home_dir)?;

    let repo_dir = get_repo_dir()?;
    assert_repo_exists(&repo_dir)?;

    let ignore_file = repo_dir.join(".gitignore");
    let ignore_patterns = get_patterns_from_file(&ignore_file);

    let config_file = get_config_file()?;
    let config_patterns = get_patterns_from_file(&config_file);

    let repo = Slice::new(repo_dir, config_patterns.clone(), ignore_patterns.clone());
    let home = Slice::new(home_dir, config_patterns.clone(), ignore_patterns.clone());

    let dotr = Dotr::new(repo, home)?;

    let cli = Cli::parse();
    match &cli.command {
        Commands::Sync => {
            dotr.sync()?;
        }
        Commands::Install => {
            dotr.install()?;
        }
        Commands::Pwd => {
            println!("{}", get_repo_dir()?.display());
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
    /// Updates the Dotr repository with all tracked files
    Sync,
    /// Prints the Dotr repository directory location
    Pwd,
    /// Lists all tracked files
    Ls,
    /// Places all tracked files into their destination
    Install,
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
