use std::{
    path::PathBuf,
    fs::{remove_file, copy, OpenOptions, read_to_string, create_dir_all, File},
    io::{Write, self}, env::{self, current_dir}
};
use glob::{Pattern, glob};
use path_absolutize::Absolutize;

use crate::prelude::*;

pub mod error;
pub mod prelude;

pub struct Dir {
    pub root_path: PathBuf,
    pub tracked_files: Vec<TrackedFile>,
}

fn run_in_dir<T, F: FnOnce() -> T>(dir: &PathBuf, f: F) -> io::Result<T> {
    let current_dir = env::current_dir()?;
    env::set_current_dir(&dir)?;
    let result = f();
    env::set_current_dir(&current_dir)?;
    Ok(result)
}

fn get_tracked_files(root_path: &PathBuf, include_patterns: &Vec<String>, exclude_patterns: &Vec<String>) -> Result<Vec<TrackedFile>> {
    println!("Getting currently tracked files in {}", root_path.display());
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
                    println!("\tCurrently ignoring file {}", &relative_path.display());
                } else {
                    println!("\tCurrently tracking file {}", &relative_path.display());
                    files.push(to_tracked_file(&relative_path, &root_path));
                }
            }
        }
        Ok(files)
    })?
}

pub struct TrackedFile {
    pub absolute_path: PathBuf,
    pub relative_path: PathBuf,
}

pub fn to_tracked_file(relative_path: &PathBuf, root_path: &PathBuf)-> TrackedFile {
    TrackedFile {
        absolute_path: root_path.join(&relative_path),
        relative_path: relative_path.into(),
    }
}

pub struct Dotr {
    repo_dir: PathBuf,
    home_dir: PathBuf,
    config_dir: PathBuf,
}

fn get_patterns_from_file(file_path: &PathBuf) -> Vec<String> {
    if let Ok(lines) = read_to_string(file_path) {
        return lines.lines().map(|l| l.to_string()).collect();
    };
    vec![]
}

fn get_or_create_config_file(config_dir: &PathBuf) -> Result<PathBuf> {
    let config_file = config_dir.join("dotr.config");
    if !config_file.exists() {
        create_dir_all(config_dir)?;
        File::create(&config_file)?;
        println!("Created config file at {}", config_file.display());
        println!("Add globs to it to start syncing your files");
    }
    Ok(config_file.to_owned())
}

impl Dotr {
    pub fn new(home_dir: PathBuf, repo_dir: PathBuf, config_dir: PathBuf) -> Self {
        println!("Creating new dotr instance");
        println!("home_dir: {}", home_dir.display());
        println!("repo_dir: {}", repo_dir.display());
        println!("config_dir: {}", config_dir.display());
        Self {
            home_dir,
            repo_dir,
            config_dir,
        }
    }

    pub fn sync(&self) -> Result<()> {
        let (repo, home) = self.get_repo_and_home()?;

        let repo_files = repo.tracked_files;
        for file in repo_files {
            println!("Removing file {}", &file.absolute_path.display());
            remove_file(&file.absolute_path).map_err(|_| Error::Generic("Failed to remove file".into()))?;
        }
        let home_files = home.tracked_files;
        for file in home_files {
            let destination = repo.root_path.join(&file.relative_path);
            let Some(parent_dir) = destination.parent() else {
                return Err(Error::Generic("Path without parent".into()));
            };
            create_dir_all(&parent_dir)
                .map_err(|_| Error::Generic("Failed to create parent dir".into()))?;
            copy(&file.absolute_path, &destination)
                .map_err(|_| Error::Generic("Failed to copy file".into()))?;
        }
        Ok(())
    }

    fn get_repo_and_home(&self) -> Result<(Dir, Dir)> {
        let include_file = get_or_create_config_file(&self.config_dir)?;
        let ignore_file = &self.repo_dir.join(".gitignore");
        let include_patterns = get_patterns_from_file(&include_file);
        let ignore_patterns = get_patterns_from_file(&ignore_file);
        let mut exclude_patterns = vec![".git/**".to_string(), ".gitignore".to_string()];
        exclude_patterns.extend_from_slice(&ignore_patterns);
        let tracked_files_in_repo = get_tracked_files(&self.repo_dir, &include_patterns, &exclude_patterns)?;
        let repo = Dir { root_path: self.repo_dir.to_owned(), tracked_files: tracked_files_in_repo };
        let tracked_files_in_home = get_tracked_files(&self.home_dir, &include_patterns, &exclude_patterns)?;
        let home = Dir { root_path: self.home_dir.to_owned(), tracked_files: tracked_files_in_home };
        Ok((repo, home))
    }

    // pub fn force_install(&self) -> Result<()> {
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

    pub fn install(&self) -> Result<()> {
        let (repo, home) = self.get_repo_and_home()?;
        let repo_files = repo.tracked_files;
        for file in repo_files {
            let destination = home.root_path.join(&file.relative_path);
            if !destination.exists() {
                let Some(parent_dir) = destination.parent() else {
                    return Err(Error::Generic("Path without parent".into()));
                };
                println!("Copying file {} to {}", file.absolute_path.display(), destination.display());
                create_dir_all(&parent_dir)
                    .map_err(|_| Error::Generic("Failed to create parent dir".into()))?;
                copy(&file.absolute_path, &destination)
                    .map_err(|_| Error::Generic("Failed to copy file".into()))?;
            }
            println!("{} already exists, skipping copy", file.relative_path.display());
        }
        Ok(())
    }

    pub fn add(&self, paths: &Vec<PathBuf>) -> Result<()> {
        let config_file_path = get_or_create_config_file(&self.config_dir)?;
        let mut config_file = OpenOptions::new()
            .read(true)
            .append(true)
            .open(&config_file_path)
            .map_err(|_| Error::Generic("Failed to open config file".into()))?;

        let config_file_lines = read_to_string(&config_file_path)
            .map_err(|_| Error::Generic("Failed to read config file".into()))?;

        for path in paths {
            println!("path {}", path.display());
            println!("current_dir {}", current_dir().unwrap().display());
            println!("home_dir {}", &self.home_dir.display());
            let relative_path = path.absolutize()
                .map_err(|_| Error::Generic("Failed to absolutize path".into()))?
                .strip_prefix(&self.home_dir)
                .map_err(|_| Error::Generic("Failed to strip home dir".into()))?
                .to_owned();
            
            println!("relpath {}", relative_path.display());

            let Some(glob) = relative_path.to_str() else {
                return Err(Error::Generic("Failed to convert path to string".into()));
            };

            if !config_file_lines.split("\n").any(|line| line == glob) {
                println!("glob added {}", glob);
                writeln!(config_file, "{}", glob).map_err(|_| Error::Generic("Failed to write to config file".into()))?;
            }
        }
        Ok(())
    }
}
