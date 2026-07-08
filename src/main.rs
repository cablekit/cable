mod build;
mod config;
mod content;
mod errors;
mod fs;
mod markdown;
mod render;
mod routes;
mod validate;

use clap::{Parser, Subcommand};
use errors::BuildError;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// A program to generate blog as code
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    ///Builds a directory
    #[command(arg_required_else_help = true)]
    Build {
        #[arg(long)]
        root: String,
    },
    ///Validates the root
    #[command(arg_required_else_help = true)]
    Validate {
        #[arg(long)]
        root: String,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");

            if let Some(source) = error.source() {
                eprintln!("Caused by: {source}");
            }

            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), BuildError> {
    let args = Args::parse();

    match args.command {
        Commands::Build { root } => {
            let root = Path::new(&root);
            let result = build::build_site(PathBuf::from(root))?;

            println!(
                "Build Result: output={}, posts={}, drafts={}, copied_assets={}",
                result.output_dir.display(),
                result.posts,
                result.drafts,
                result.copied_assets
            );

            Ok(())
        }
        Commands::Validate { root } => {
            let root = Path::new(&root);

            let result = validate::validate_build(PathBuf::from(root))?;

            println!(
                "Validate Result: posts={}, drafts={}",
                result.posts, result.drafts
            );

            Ok(())
        }
    }
}
