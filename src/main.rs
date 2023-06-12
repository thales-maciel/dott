use git2::Repository;
use std::fs::create_dir_all;

use directories::ProjectDirs;
use clap::{Args, Parser, Subcommand};

pub fn init() {
    if let Some(proj_dirs) = ProjectDirs::from("dev", "thales-maciel", "dotr") {
        let data_dir = proj_dirs.data_dir();
        println!("data dir is {:?}", data_dir);
        if !data_dir.exists() {
            if let Err(_e) = create_dir_all(data_dir) {
                println!("Failed to create data dir {:?}", _e);
            }
            if let Err(_e) = Repository::init(data_dir) {
                println!("Failed to initialize repository {:?}", _e);
            }
        } else {
            // check if it's already initialized


        }
    }
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Init => {
            init();
        },
        _ => {}
    }
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

