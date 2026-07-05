use crate::errors::BuildError;
use chrono::NaiveDate;
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

fn post_from_parsed_file(parsed_file: Parsed, source_path: &PathBuf) -> Result<Post, BuildError> {
    let frontmatter: PostFrontmatter =
        serde_yaml::from_str(&parsed_file.frontmatter).map_err(|source| {
            BuildError::ParseFrontmatter {
                path: source_path.clone(),
                source,
            }
        })?;
    let date = NaiveDate::parse_from_str(&frontmatter.date, "%Y-%m-%d").map_err(|source| {
        BuildError::ParseDate {
            path: source_path.clone(),
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
        source_path: source_path.clone(),
        output_path: PathBuf::new(),
        url: format!("/{}/", frontmatter.slug),
    })
}
