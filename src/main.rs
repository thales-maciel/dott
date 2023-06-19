use dotr::{
    Dotr,
    prelude::*,
};

use clap::{Args, Parser, Subcommand};
use directories::{BaseDirs, ProjectDirs};
use std::path::PathBuf;

fn get_repo_dir() -> Result<PathBuf> {
    let Some(project_dir) = ProjectDirs::from("dev", "thales-maciel", "dotr") else {
        return Err(Error::Generic("No valid path could be retrieved from system".into()))
    };
    Ok(project_dir.data_dir().to_owned())
}

fn get_home_dir() -> Result<PathBuf> {
    let Some(base_dir) = BaseDirs::new() else {
        return Err(Error::Generic("No valid path could be retrieved from system".into()))
    };
    Ok(base_dir.home_dir().to_owned())
}

fn get_config_dir() -> Result<PathBuf> {
    let Some(project_dir) = ProjectDirs::from("dev", "thales-maciel", "dotr") else {
        return Err(Error::Generic("No valid path could be retrieved from system".into()))
    };
    Ok(project_dir.config_dir().to_owned())
}

fn main() -> Result<()> {
    let home_dir = get_home_dir()?;
    let repo_dir = get_repo_dir()?;
    let config_dir = get_config_dir()?;
    let dotr = Dotr::new(home_dir, repo_dir, config_dir);

    let cli = Cli::parse();
    match &cli.command {
        Commands::Sync => {
            dotr.sync()?;
        }
        Commands::Install(args) => {
            dotr.install(args.force)?;
        }
        Commands::Pwd => {
            println!("{}", get_repo_dir()?.display());
        }
        Commands::Add(args) => {
            dotr.add(&args.paths)?;
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
    Add(AddArgs),
    /// Updates the Dotr repository with all tracked files
    Sync,
    /// Prints the Dotr repository directory location
    Pwd,
    /// Places all tracked files into their destination
    Install(InstallArgs),
}

#[derive(Args)]
pub struct InstallArgs {
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Args)]
pub struct AddArgs {
    pub paths: Vec<PathBuf>,
}
