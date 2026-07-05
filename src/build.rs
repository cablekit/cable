use std::cmp::Reverse;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::content::{Post, Status};
use crate::errors::BuildError;
use crate::{config, content, fs, markdown, render, routes};

#[derive(Debug)]
pub struct BuildResult {
    pub output_dir: PathBuf,
    pub posts: usize,
    pub drafts: usize,
    pub copied_assets: usize,
}

pub fn build_site(root: PathBuf) -> Result<BuildResult, BuildError> {
    println!("Building site at: {:?}", root);

    let config_path = root.join("blog.toml");
    let config = config::load_config(config_path)?;

    let posts_dir = root.join(&config.content.posts);
    let output_dir = root.join(&config.output.directory);
    let public_dir = root.join("public");

    fs::clean_dir(&output_dir)?;
    fs::ensure_dir(&output_dir)?;

    //Non-recursive copy of assets. This skips over any directories
    let copied_assets = if public_dir.exists() {
        fs::copy_dir_contents(&public_dir, &output_dir)?
    } else {
        0
    };

    let source_files = content::discover_markdown_files(&posts_dir)?;

    let mut posts: Vec<Post> = Vec::new();

    for source_path in source_files {
        let post = content::post_from_path_location(&source_path)?;
        posts.push(post);
    }

    validate_duplicate_slugs(&posts)?;

    for post in posts.iter_mut() {
        post.html = markdown::to_html(&post.body)?;

        post.url = routes::post_url(
            &config.routes.post,
            &post.slug,
            &posts_dir,
            &post.source_path,
        )?;

        post.output_path = routes::post_output_path(&output_dir, &post.url);
    }

    let drafts = posts
        .iter()
        .filter(|post| post.status == Status::Draft)
        .count();

    let mut published_posts = posts
        .into_iter()
        .filter(|post| post.status == Status::Published)
        .collect::<Vec<_>>();

    published_posts.sort_by_key(|post| Reverse(post.date));

    let index_html = render::render_index(&config, &published_posts)?;

    let index_path = output_dir.join("index.html");

    fs::write_file(&index_path, &index_html)?;

    for post in &published_posts {
        let post_html = render::render_post(&config, post)?;

        fs::write_file(&post.output_path, &post_html)?;
    }

    Ok(BuildResult {
        output_dir: root.join("dist"),
        posts: published_posts.len(),
        drafts,
        copied_assets,
    })
}

fn validate_duplicate_slugs(posts: &[Post]) -> Result<(), BuildError> {
    let mut seen: HashMap<String, PathBuf> = HashMap::new();

    for post in posts {
        if let Some(previous_path) = seen.get(&post.slug) {
            return Err(BuildError::DuplicateSlug {
                slug: post.slug.clone(),
                first_path: previous_path.clone(),
                second_path: post.source_path.clone(),
            });
        }

        seen.insert(post.slug.clone(), post.source_path.clone());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_dir(name: &str) -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("cable-{name}-{id}"))
    }

    fn post(slug: &str, source_path: PathBuf) -> Post {
        Post {
            title: slug.to_string(),
            date: NaiveDate::from_ymd_opt(2026, 7, 5).unwrap(),
            slug: slug.to_string(),
            tags: Vec::new(),
            status: Status::Published,
            body: String::new(),
            html: String::new(),
            source_path,
            output_path: PathBuf::new(),
            url: String::new(),
        }
    }

    #[test]
    fn validate_duplicate_slugs_reports_both_paths() {
        let posts = vec![
            post("hello", PathBuf::from("content/posts/one.md")),
            post("hello", PathBuf::from("content/posts/two.md")),
        ];

        let error = validate_duplicate_slugs(&posts).unwrap_err();

        assert!(matches!(
            error,
            BuildError::DuplicateSlug {
                ref slug,
                ref first_path,
                ref second_path
            } if slug == "hello"
                && first_path == &PathBuf::from("content/posts/one.md")
                && second_path == &PathBuf::from("content/posts/two.md")
        ));
    }

    #[test]
    fn build_site_writes_index_and_nested_post() {
        let root = test_dir("build");
        let posts_dir = root.join("content").join("posts");
        fs::create_dir_all(posts_dir.join("nested")).unwrap();
        fs::write(
            root.join("blog.toml"),
            r#"
[site]
title = "Test Blog"
description = "A test site"
url = "https://example.com"

[content]
posts = "content/posts"

[output]
directory = "dist"

[routes]
post = "/posts/:slug"
"#,
        )
        .unwrap();
        fs::write(
            posts_dir.join("nested").join("hello.md"),
            r#"---
title: "Hello"
date: "2026-07-05"
slug: "hello"
tags: []
status: "published"
---

# Hello

Nested body.
"#,
        )
        .unwrap();

        let result = build_site(root.clone()).unwrap();

        assert_eq!(result.posts, 1);
        assert_eq!(result.drafts, 0);
        assert!(root.join("dist").join("index.html").exists());
        assert!(
            root.join("dist")
                .join("posts")
                .join("nested")
                .join("hello.html")
                .exists()
        );
        fs::remove_dir_all(root).unwrap();
    }
}
