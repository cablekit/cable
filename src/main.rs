mod build;
mod config;
mod content;
mod errors;
mod feed;
mod fs;
mod markdown;
mod render;
mod routes;

use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    }
}
