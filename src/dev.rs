use crate::build;
use crate::errors::BuildError;
use ascii::AsciiString;
use notify_debouncer_mini::new_debouncer;
use notify_debouncer_mini::notify::RecursiveMode;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

pub fn serve(root: &Path, port: u16) -> Result<(), BuildError> {
    let server_address = format!("127.0.0.1:{}", port);
    println!("Starting Dev server for {:?} at {}", root, server_address);

    let server = tiny_http::Server::http(server_address).unwrap();
    let result = build::build_site(PathBuf::from(root))?;
    let output_dir = result.output_dir.clone();
    watch_for_changes(root.to_path_buf());

    loop {
        let rq = match server.recv() {
            Ok(rq) => rq,
            Err(_) => break,
        };

        let url = rq.url().to_string();
        let url_path = url.split('?').next().unwrap_or("/");
        let relative_path = url_path.trim_start_matches('/');
        let path = if relative_path.is_empty() {
            output_dir.join("index.html")
        } else {
            output_dir.join(relative_path)
        };
        let file = std::fs::File::open(&path);

        if file.is_ok() {
            let response = tiny_http::Response::from_file(file.unwrap());

            let response = response.with_header(tiny_http::Header {
                field: "Content-Type".parse().unwrap(),
                value: AsciiString::from_ascii(get_content_type(&path)).unwrap(),
            });

            let _ = rq.respond(response);
        } else {
            let rep = tiny_http::Response::new_empty(tiny_http::StatusCode(404));
            let _ = rq.respond(rep);
        }
    }

    Ok(())
}

fn watch_for_changes(root: PathBuf) {
    thread::spawn(move || {
        let (tx, rx) = mpsc::channel();
        let mut debouncer = new_debouncer(Duration::from_millis(500), tx).unwrap();

        debouncer
            .watcher()
            .watch(&root.join("blog.toml"), RecursiveMode::NonRecursive)
            .unwrap();

        let content_dir = root.join("content");
        if content_dir.exists() {
            debouncer
                .watcher()
                .watch(&content_dir, RecursiveMode::Recursive)
                .unwrap();
        }

        let rebuild_cooldown = Duration::from_secs(1);
        let mut last_rebuild = Instant::now() - rebuild_cooldown;

        for events in rx {
            match events {
                Ok(events) if !events.is_empty() => {
                    let now = Instant::now();
                    if now.duration_since(last_rebuild) < rebuild_cooldown {
                        continue;
                    }
                    last_rebuild = now;

                    println!("Changes detected, rebuilding site");
                    if let Err(error) = build::build_site(root.clone()) {
                        eprintln!("Rebuild failed: {error}");
                    }
                }
                Ok(_) => {}
                Err(error) => eprintln!("Watcher error: {error:?}"),
            }
        }
    });
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
