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


fn main() {
    let args = Args::parse();

    match args.command{
        Commands::Build {root} => {
            println!("Building from root: {}", root)
        }
    }
}
