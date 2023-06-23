use dotr::{prelude::*, sync_dirs};

use clap::{Args, Parser, Subcommand};
use directories::BaseDirs;
use std::path::PathBuf;

fn get_home_dir() -> Result<PathBuf> {
    let Some(base_dir) = BaseDirs::new() else {
        return Err(DotrError::Generic("No valid path could be retrieved from system".into()))
    };
    Ok(base_dir.home_dir().to_owned())
}

fn main() -> Result<()> {
    let cwd = std::env::current_dir()?;

    let cli = Cli::parse();
    match &cli.command {
        Commands::Install(args) => {
            let source = &cwd;
            let patterns_file = &cwd.join("dotr.config");
            let target = get_home_dir()?;
            let raw = &args.raw;

            println!("Will use patterns from {}", patterns_file.display());
            println!("To gather files from {}", source.display());
            println!("And sync them to {}", target.display());
            println!();

            if let Err(e) = sync_dirs(&patterns_file, &source, &target, raw) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
        Commands::Refresh(args) => {
            let target = &cwd;
            let patterns_file = &cwd.join("dotr.config");
            let source = get_home_dir()?;
            let raw = &args.raw;

            println!("Will use patterns from {}", patterns_file.display());
            println!("To gather files from {}", source.display());
            println!("And sync them to {}", target.display());
            println!();

            if let Err(e) = sync_dirs(&patterns_file, &source, &target, raw) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
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
    /// Pulls files from home into the current directory.
    Refresh(SyncArgs),
    /// Places all tracked files into their destination
    Install(SyncArgs),
}

#[derive(Args)]
pub struct SyncArgs {
    #[arg(short, long, default_value_t = false)]
    pub raw: bool,
}

#[derive(Args)]
pub struct InstallArgs {
    #[arg(short, long)]
    pub force: bool,
}
