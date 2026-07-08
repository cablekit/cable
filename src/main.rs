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
use ascii::AsciiString;
use serde::de::Expected;

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
    ///Dev Server
    #[command(arg_required_else_help = true)]
    Dev {
        #[arg(long)]
        root: String,
        #[arg(long, default_value_t = 3119)]
        port: u16,
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
        },
        Commands::Dev {root, port} => {
            let root = Path::new(&root);
            let server_address = format!("127.0.0.1:{}", port);
            println!("Starting Dev server for {:?} at {}", root, server_address);

            let server = tiny_http::Server::http(server_address).unwrap();
            let result = build::build_site(PathBuf::from(root))?;
            let output_dir = &result.output_dir;
            loop{
                let rq = match server.recv(){
                    Ok(rq)=> rq,
                    Err(_) => break
                };

                println!("{:?}", rq);


                let url = rq.url().to_string();
                let url_path = url.split('?').next().unwrap_or("/");
                let relative_path = url_path.trim_start_matches('/');
                let path = if relative_path.is_empty() {
                    output_dir.join("index.html")
                } else {
                    output_dir.join(relative_path)
                };
                let file = std::fs::File::open(&path);

                println!("Build Result {:#?} \n URL: {} \n Path {:?} \n File {:?}", result, url, path,file);

                if file.is_ok() {
                    let response = tiny_http::Response::from_file(file.unwrap());

                    let response = response.with_header(tiny_http::Header {
                        field: "Content-Type".parse().unwrap(),
                        value: AsciiString::from_ascii(get_content_type(&path)).unwrap(),
                    });

                    let _ = rq.respond(response);
                }else{
                    let rep = tiny_http::Response::new_empty(tiny_http::StatusCode(404));
                    let _ = rq.respond(rep);
                }
            }

            Ok(())
        }
    }
}


fn get_content_type(path: &Path) -> &'static str {
    let extension = match path.extension() {
        None => return "text/plain",
        Some(e) => e,
    };

    match extension.to_str().unwrap() {
        "gif" => "image/gif",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        "png" => "image/png",
        "pdf" => "application/pdf",
        "htm" => "text/html; charset=utf8",
        "html" => "text/html; charset=utf8",
        "txt" => "text/plain; charset=utf8",
        "css" => "text/css; charset=utf8",
        _ => "text/plain; charset=utf8",
    }
}