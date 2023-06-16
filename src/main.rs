use crate::prelude::*;

mod error;
mod prelude;

use clap::{Args, Parser, Subcommand};
use directories::{BaseDirs, ProjectDirs};
use git2::Repository;
use glob::{glob, Pattern};
use std::{
    env,
    fs::{create_dir_all, read_to_string, remove_file, copy},
    path::PathBuf, io,
};

pub struct Dir {
    root_path: PathBuf,
    tracked_files: Vec<TrackedFile>,
}

struct TrackedFile {
    absolute_path: PathBuf,
    relative_path: PathBuf,
}

fn to_tracked_file(relative_path: &PathBuf, root_path: &PathBuf)-> TrackedFile {
    TrackedFile {
        absolute_path: root_path.join(&relative_path),
        relative_path: relative_path.into(),
    }
}

struct Dotr {
    repo: Dir,
    home: Dir,
}

impl Dotr {
    fn new(repo: Dir, home: Dir) -> Self {
        Self {
            repo,
            home,
        }
    }

    fn sync(&self) -> Result<()> {
        let repo_files = &self.repo.tracked_files;
        for file in repo_files {
            println!("Removing file {}", &file.absolute_path.display());
            remove_file(&file.absolute_path).map_err(|_| Error::Generic("Failed to remove file".into()))?;
        }
        let home_files = &self.home.tracked_files;
        for file in home_files {
            let destination = self.repo.root_path.join(&file.relative_path);
            println!("Copying file {} to {}", file.absolute_path.display(), destination.display());
            match copy(&file.absolute_path, destination) {
                Ok(_) => Ok(()),
                Err(_) => Err(Error::Generic("Failed to copy file".into())),
            }?;
        }
        Ok(())
    }

    // fn force_install(&self) -> Result<()> {
    //     let repo_files = &self.repo.tracked_files;
    //     for file in repo_files {
    //         let destination = self.home.root_path.join(&file.relative_path);
    //         println!("Copying file {} to {}", file.absolute_path.display(), destination.display());
    //         match copy(&file.absolute_path, destination) {
    //             Ok(_) => Ok(()),
    //             Err(_) => Err(Error::Generic("Failed to copy file".into())),
    //         }?;
    //     }
    //     Ok(())
    // }

    fn install(&self) -> Result<()> {
        let repo_files = &self.repo.tracked_files;
        for file in repo_files {
            let destination = self.home.root_path.join(&file.relative_path);
            if !destination.exists() {
                println!("Copying file {} to {}", file.absolute_path.display(), destination.display());
                match copy(&file.absolute_path, destination) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(Error::Generic("Failed to copy file".into())),
                }?;
            }
            println!("{} already exists, skipping copy", file.relative_path.display());
        }
        Ok(())
    }
}

fn run_in_dir<T, F: FnOnce() -> T>(dir: &PathBuf, f: F) -> io::Result<T> {
    let current_dir = env::current_dir()?;
    env::set_current_dir(&dir)?;
    let result = f();
    env::set_current_dir(&current_dir)?;
    Ok(result)
}

fn get_tracked_files(root_path: &PathBuf, include_patterns: &Vec<String>, exclude_patterns: &Vec<String>) -> Result<Vec<TrackedFile>> {
    println!("Getting tracked files for {}", root_path.display());
    run_in_dir(&root_path, || {
        let mut ignore_patterns = Vec::new();
        for p in exclude_patterns {
            ignore_patterns.push(Pattern::new(p).map_err(|_| Error::Generic("Bad pattern".into()))?);
        }

        let mut files = Vec::new();
        for pattern in include_patterns {
            let paths = glob(&pattern).map_err(|_| Error::Generic("Bad pattern".into()))?;
            for path in paths {
                let relative_path = path.map_err(|_| Error::Generic("Bad pattern".into()))?;
                if ignore_patterns.iter().any(|p| p.matches(relative_path.to_str().unwrap())) {
                    println!("Ignoring file {}", &root_path.join(&relative_path).display());
                } else {
                    println!("Added file {}", &root_path.join(&relative_path).display());
                    files.push(to_tracked_file(&relative_path, &root_path));
                }
            }
        }
        Ok(files)
    })?
}

fn assert_repo_exists(dir: &PathBuf) -> Result<()> {
    if let Err(_) = create_dir_all(&dir) {
        return Err(Error::Generic("Failed to create data directory".into()));
    };

    match Repository::init(dir) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::Generic("Failed to initialize repository".into())),
    }
}

fn get_patterns_from_file(file_path: &PathBuf) -> Vec<String> {
    if let Ok(lines) = read_to_string(file_path) {
        return lines.lines().map(|l| l.to_string()).collect();
    };
    vec![]
}

fn get_repo_dir() -> Result<PathBuf> {
    let Some(project_dir) = ProjectDirs::from("dev", "thales-maciel", "dotr") else {
        return Err(Error::Generic("No valid path could be retrieved from system".into()))
    };
    Ok(project_dir.data_dir().to_owned())
}

// todo: make this a get_or_create instead
fn get_config_file() -> Result<PathBuf> {
    let Some(project_dir) = ProjectDirs::from("dev", "thales-maciel", "dotr") else {
        return Err(Error::Generic("No valid path could be retrieved from system".into()))
    };
    Ok(project_dir.config_dir().join("dotr.config"))
}

fn get_home_dir() -> Result<PathBuf> {
    let Some(base_dir) = BaseDirs::new() else {
        return Err(Error::Generic("No valid path could be retrieved from system".into()))
    };
    Ok(base_dir.home_dir().to_owned())
}

fn build_dotr() -> Result<Dotr> {
    let home_dir = get_home_dir()?;
    let repo_dir = get_repo_dir()?;
    assert_repo_exists(&repo_dir)?;

    let include_file = get_config_file()?;
    let ignore_file = repo_dir.join(".gitignore");

    let include_patterns = get_patterns_from_file(&include_file);
    let ignore_patterns = get_patterns_from_file(&ignore_file);

    let mut exclude_patterns = vec![".git/**".to_string(), ".gitignore".to_string()];
    exclude_patterns.extend_from_slice(&ignore_patterns);

    let tracked_files_in_repo = get_tracked_files(&repo_dir, &include_patterns, &exclude_patterns)?;
    let repo = Dir { root_path: repo_dir, tracked_files: tracked_files_in_repo };

    let tracked_files_in_home = get_tracked_files(&home_dir, &include_patterns, &exclude_patterns)?;
    let home = Dir { root_path: home_dir, tracked_files: tracked_files_in_home };

    let dotr = Dotr::new(repo, home);
    Ok(dotr)
}


fn main() -> Result<()> {
    let dotr = build_dotr()?;

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
    // Add(AddArgs),
    /// Updates the Dotr repository with all tracked files
    Sync,
    /// Prints the Dotr repository directory location
    Pwd,
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
