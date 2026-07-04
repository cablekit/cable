mod build;
mod config;
mod fs;

use std::path::{Path, PathBuf};
use clap::{Parser, Subcommand};

/// A program to generate blog as code
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands{
    ///Builds a directory
    #[command(arg_required_else_help = true)]
    Build {
        #[arg(
            long
        )]
        root: String
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command{
        Commands::Build {root} => {
            let root = Path::new(&root);
            let result = build::build_site(PathBuf::from(root))?;

            println!("Build Result {:#?}", result);

            Ok(())

        }
    }
}
