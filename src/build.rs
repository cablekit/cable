use std::cmp::Reverse;
use std::path::PathBuf;

use crate::content::Status;
use crate::errors::BuildError;
use crate::{fs, markdown, render, routes, tags, validate};

#[derive(Debug)]
pub struct BuildResult {
    pub output_dir: PathBuf,
    pub posts: usize,
    pub drafts: usize,
    pub copied_assets: usize,
}

pub fn build_site(root: PathBuf) -> Result<BuildResult, BuildError> {
    println!("Building site at: {}", root.display());

    let site = validate::validate_site(root.clone())?;
    let config = site.config;
    let posts_dir = site.posts_dir;
    let output_dir = site.output_dir;
    let public_dir = root.join("public");

    fs::clean_dir(&output_dir)?;
    fs::ensure_dir(&output_dir)?;

    //Copy in the default CSS the user overides.
    //Should put in Readme this poss
    fs::write_file(&output_dir.join("style.css"), render::default_css())?;

    //Non-recursive copy of assets. This skips over any directories
    let copied_assets = if public_dir.exists() {
        fs::copy_dir_contents(&public_dir, &output_dir)?
    } else {
        0
    };

    let mut posts = site.posts;

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

    let mut published_posts = posts
        .into_iter()
        .filter(|post| post.status == Status::Published)
        .collect::<Vec<_>>();

    published_posts.sort_by_key(|post| Reverse(post.date));

    let tags = tags::generate_tags(&published_posts);
    let tag_root = output_dir.join("tags");

    let index_html = render::render_index(&config, &published_posts)?;
    let index_path = output_dir.join("index.html");

    fs::write_file(&index_path, &index_html)?;

    for post in &published_posts {
        let post_html = render::render_post(&config, post)?;

        fs::write_file(&post.output_path, &post_html)?;
    }

    for tag in &tags {
        let tag_html = render::render_tag(&config, tag)?;
        let tag_path = tag_root.join(format!("{}.html", tag.slug));

        fs::write_file(&tag_path, &tag_html)?;
    }

    Ok(BuildResult {
        output_dir,
        posts: published_posts.len(),
        drafts: site.drafts,
        copied_assets,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_dir(name: &str) -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("cable-{name}-{id}"))
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
post = "./posts/:slug"
"#,
        )
        .unwrap();
        fs::write(
            posts_dir.join("nested").join("hello.md"),
            r#"---
title: "Hello"
date: "2026-07-05"
slug: "hello"
tags:
  - rust
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
        let tag_page_path = root.join("dist").join("tags").join("rust.html");
        assert!(tag_page_path.exists());

        let tag_page = fs::read_to_string(tag_page_path).unwrap();
        assert!(tag_page.contains("<h1>rust</h1>"));
        assert!(tag_page.contains(r#"<a href="/posts/nested/hello.html">Hello</a>"#));
        assert!(!tag_page.contains(r#"href="./posts/nested/hello.html""#));

        fs::remove_dir_all(root).unwrap();
    }
}
