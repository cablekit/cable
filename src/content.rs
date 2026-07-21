use crate::errors::BuildError;
use crate::{config, fs as cable_fs};
use chrono::{Local, NaiveDate};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

const MARKDOWN_EXTENSION: &str = "md";

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Published,
    Draft,
}

#[derive(Debug)]
pub struct Post {
    pub(crate) title: String,
    pub(crate) date: NaiveDate,
    pub slug: String,
    pub(crate) tags: Vec<String>,
    pub(crate) status: Status,
    pub body: String,
    pub html: String,
    pub source_path: PathBuf,
    pub output_path: PathBuf,
    pub(crate) url: String,
}

#[derive(Debug)]
struct Parsed {
    frontmatter: String,
    body: String,
}

#[derive(Deserialize)]
struct PostFrontmatter {
    title: String,
    date: String,
    slug: String,
    tags: Vec<String>,
    status: Status,
}

pub fn discover_markdown_files(directory: &Path) -> Result<Vec<PathBuf>, BuildError> {
    let mut files: Vec<PathBuf> = Vec::new();

    for entry in fs::read_dir(directory).map_err(|source| BuildError::ReadDirectory {
        path: directory.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| BuildError::ReadDirectoryEntry {
            directory: directory.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let ty = entry
            .file_type()
            .map_err(|source| BuildError::ReadDirectoryEntry {
                directory: directory.to_path_buf(),
                source,
            })?;

        if ty.is_dir() {
            files.extend(discover_markdown_files(&path)?);
        } else if path
            .extension()
            .is_some_and(|extension| extension == MARKDOWN_EXTENSION)
        {
            files.push(path);
        }
    }

    Ok(files)
}

pub fn post_from_path_location(source_path: &PathBuf) -> Result<Post, BuildError> {
    let raw_file = read_markdown_file(source_path)?;
    let parsed_file = parse_post_file(&raw_file, source_path)?;
    let post = post_from_parsed_file(parsed_file, source_path)?;
    Ok(post)
}

pub fn create_new_post(root: &Path, title: &str) -> Result<PathBuf, BuildError> {
    let config = config::load_config(root.join("blog.toml"))?;
    let date = Local::now().date_naive();
    let slug = title_to_slug(title);
    let file_name = format!("{}-{slug}.md", date.format("%Y-%m-%d"));
    let post_path = root.join(config.content.posts).join(file_name);
    let content = new_post_content(title, &slug, &date.to_string());

    cable_fs::write_file(&post_path, &content)?;

    Ok(post_path)
}

fn read_markdown_file(source_path: &PathBuf) -> Result<String, BuildError> {
    let content = fs::read_to_string(source_path).map_err(|source| BuildError::ReadFile {
        path: source_path.clone(),
        source,
    })?;
    //Normalizes to Unix
    let content = content.replace("\r\n", "\n");
    Ok(content)
}

fn parse_post_file(raw_file: &str, source_path: &Path) -> Result<Parsed, BuildError> {
    let raw_file = raw_file.strip_prefix('\u{feff}').unwrap_or(raw_file);
    let raw_file =
        raw_file
            .strip_prefix("---\n")
            .ok_or_else(|| BuildError::MissingOpeningFrontmatter {
                path: source_path.to_path_buf(),
            })?;

    let (frontmatter, body) =
        raw_file
            .split_once("\n---\n")
            .ok_or_else(|| BuildError::MissingClosingFrontmatter {
                path: source_path.to_path_buf(),
            })?;

    Ok(Parsed {
        frontmatter: frontmatter.to_string(),
        body: body.to_string(),
    })
}

fn post_from_parsed_file(parsed_file: Parsed, source_path: &Path) -> Result<Post, BuildError> {
    let frontmatter: PostFrontmatter =
        serde_yaml::from_str(&parsed_file.frontmatter).map_err(|source| {
            BuildError::ParseFrontmatter {
                path: source_path.to_path_buf(),
                source,
            }
        })?;
    let date = NaiveDate::parse_from_str(&frontmatter.date, "%Y-%m-%d").map_err(|source| {
        BuildError::ParseDate {
            path: source_path.to_path_buf(),
            date: frontmatter.date.clone(),
            source,
        }
    })?;

    Ok(Post {
        title: frontmatter.title,
        date,
        slug: frontmatter.slug.clone(),
        tags: frontmatter.tags,
        status: frontmatter.status,
        body: parsed_file.body,
        html: String::new(),
        source_path: source_path.to_path_buf(),
        output_path: PathBuf::new(),
        url: format!("/{}/", frontmatter.slug),
    })
}

fn new_post_content(title: &str, slug: &str, date: &str) -> String {
    format!(
        r#"---
title: "{}"
date: "{}"
slug: "{}"
tags: []
status: "draft"
---

# This is where your post goes

"#,
        yaml_escape(title),
        date,
        slug
    )
}

pub fn title_to_slug(title: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_dash = false;

    for character in title.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            previous_was_dash = false;
        } else if !previous_was_dash && !slug.is_empty() {
            slug.push('-');
            previous_was_dash = true;
        }
    }

    slug.trim_end_matches('-').to_string()
}

fn yaml_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_dir(name: &str) -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("cable-{name}-{id}"))
    }

    fn valid_post() -> String {
        String::from(
            r#"---
title: "Hello World"
date: "2026-07-05"
slug: "hello-world"
tags:
  - intro
status: "published"
---

# Hello

Body text.
"#,
        )
    }

    #[test]
    fn discover_markdown_files_finds_nested_markdown_files() {
        let root = test_dir("discover");
        let posts = root.join("posts");
        fs::create_dir_all(posts.join("nested")).unwrap();
        fs::write(posts.join("hello.md"), "").unwrap();
        fs::write(posts.join("notes.txt"), "").unwrap();
        fs::write(posts.join("nested").join("review.md"), "").unwrap();

        let mut files = discover_markdown_files(&posts).unwrap();
        files.sort();

        assert_eq!(files.len(), 2);
        assert!(files.contains(&posts.join("hello.md")));
        assert!(files.contains(&posts.join("nested").join("review.md")));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn parse_post_file_splits_frontmatter_and_body() {
        let path = PathBuf::from("hello.md");

        let parsed = parse_post_file(&valid_post(), &path).unwrap();

        assert!(parsed.frontmatter.contains("title: \"Hello World\""));
        assert!(parsed.body.contains("# Hello"));
    }

    #[test]
    fn post_from_path_location_deserializes_frontmatter() {
        let root = test_dir("post");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("hello.md");
        fs::write(&path, valid_post()).unwrap();

        let post = post_from_path_location(&path).unwrap();

        assert_eq!(post.title, "Hello World");
        assert_eq!(post.date, NaiveDate::from_ymd_opt(2026, 7, 5).unwrap());
        assert_eq!(post.slug, "hello-world");
        assert_eq!(post.status, Status::Published);
        assert!(post.body.contains("Body text."));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn invalid_frontmatter_date_returns_parse_date_error() {
        let parsed = Parsed {
            frontmatter: String::from(
                r#"title: "Bad Date"
date: "2026-13-05"
slug: "bad-date"
tags: []
status: "published"
"#,
            ),
            body: String::new(),
        };
        let path = PathBuf::from("bad-date.md");

        let error = post_from_parsed_file(parsed, &path).unwrap_err();

        assert!(matches!(error, BuildError::ParseDate { .. }));
    }

    #[test]
    fn title_to_slug_normalizes_title_for_filename() {
        assert_eq!(title_to_slug("Hello, Cable World!"), "hello-cable-world");
        assert_eq!(title_to_slug("  Multiple   Spaces  "), "multiple-spaces");
    }

    #[test]
    fn new_post_content_creates_draft_frontmatter() {
        let content = new_post_content("Hello \"Cable\"", "hello-cable", "2026-07-08");

        assert!(content.contains("title: \"Hello \\\"Cable\\\"\""));
        assert!(content.contains("date: \"2026-07-08\""));
        assert!(content.contains("slug: \"hello-cable\""));
        assert!(content.contains("tags: []"));
        assert!(content.contains("status: \"draft\""));
    }
}
