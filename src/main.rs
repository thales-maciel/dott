#![allow(unused)]

use crate::prelude::*;

mod error;
mod prelude;

use git2::Repository;
use std::fs::create_dir_all;

use directories::ProjectDirs;
use clap::{Args, Parser, Subcommand};

pub fn assert_repo_exists(dir: &str) -> Result<()> {
    match Repository::init(dir) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::Generic("Failed to initialize repository".into()))
    }
}

pub fn init() -> Result<()> {
    if let Some(proj_dirs) = ProjectDirs::from("dev", "thales-maciel", "dotr") {
        // create_dir_all(&proj_dirs.data_dir())?;
        if let Err(_) = create_dir_all(&proj_dirs.data_dir()) {
            return Err(Error::Generic("Failed to create data directory".into()));
        };

        return assert_repo_exists(proj_dirs.data_dir().to_str().unwrap());
    }

    Err(Error::Generic("Project dir not found".into()))
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let res = match &cli.command {
        Commands::Init => {
            init();
        },
        _ => {}
    };
    
    println!("res: {:#?}", res);

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
    Update,
    /// Goes to the Dotr repository directory
    Cd,
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

