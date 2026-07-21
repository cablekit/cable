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

struct BuildPage {
    url: String,
    output_path: PathBuf,
    contents: String,
    include_in_sitemap: bool,
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

    //Start Building Pages
    let mut generated_pages: Vec<BuildPage> = Vec::new();

    let tags = tags::generate_tags(&published_posts);
    let tag_root = output_dir.join("tags");

    let index_html = render::render_index(&config, &published_posts)?;
    let index_path = output_dir.join("index.html");

    generated_pages.push(BuildPage {
        url: "/".to_string(),
        output_path: index_path,
        contents: index_html,
        include_in_sitemap: true,
    });

    for post in &published_posts {
        let post_html = render::render_post(&config, post)?;

        generated_pages.push(BuildPage {
            url: post.url.clone(),
            output_path: post.output_path.clone(),
            contents: post_html,
            include_in_sitemap: true,
        });
    }

    for tag in &tags {
        let tag_html = render::render_tag(&config, tag)?;
        let tag_path = tag_root.join(format!("{}.html", tag.slug));
        let tag_url = routes::output_path_to_url(&output_dir, &tag_path)?;

        generated_pages.push(BuildPage {
            url: tag_url,
            output_path: tag_path,
            contents: tag_html,
            include_in_sitemap: true,
        });
    }

    for page in &generated_pages {
        fs::write_file(&page.output_path, &page.contents)?;
    }

    let sitemap_urls = generated_pages
        .iter()
        .filter(|page| page.include_in_sitemap)
        .map(|page| page.url.as_str());

    let sitemap_xml = render::render_sitemap(&config, sitemap_urls)?;

    fs::write_file(&output_dir.join("sitemap.xml"), &sitemap_xml)?;

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

    fn create_test_site(name: &str) -> PathBuf {
        let root = test_dir(name);
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
        fs::write(
            posts_dir.join("draft.md"),
            r#"---
title: "Draft"
date: "2026-07-06"
slug: "draft"
tags:
  - drafts
status: "draft"
---

# Draft

Draft body.
"#,
        )
        .unwrap();

        root
    }

    #[test]
    fn build_site_writes_index_and_nested_post() {
        let root = create_test_site("build-pages");

        let result = build_site(root.clone()).unwrap();

        assert_eq!(result.posts, 1);
        assert_eq!(result.drafts, 1);
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

    #[test]
    fn build_site_writes_tag_pages() {
        let root = create_test_site("build-tags");

        build_site(root.clone()).unwrap();

        let tag_page_path = root.join("dist").join("tags").join("rust.html");
        assert!(tag_page_path.exists());

        let tag_page = fs::read_to_string(tag_page_path).unwrap();
        assert!(tag_page.contains("<h1>rust</h1>"));
        assert!(tag_page.contains(r#"<a href="/posts/nested/hello.html">Hello</a>"#));
        assert!(!tag_page.contains(r#"href="./posts/nested/hello.html""#));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn build_site_writes_sitemap_for_generated_pages() {
        let root = create_test_site("build-sitemap");

        build_site(root.clone()).unwrap();

        let sitemap_path = root.join("dist").join("sitemap.xml");
        assert!(sitemap_path.exists());

        let sitemap = fs::read_to_string(sitemap_path).unwrap();
        assert!(sitemap.contains(r#"<loc>https://example.com/</loc>"#));
        assert!(sitemap.contains(r#"<loc>https://example.com/posts/nested/hello.html</loc>"#));
        assert!(sitemap.contains(r#"<loc>https://example.com/tags/rust.html</loc>"#));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn build_site_excludes_drafts_from_sitemap() {
        let root = create_test_site("build-sitemap-drafts");

        build_site(root.clone()).unwrap();

        let sitemap = fs::read_to_string(root.join("dist").join("sitemap.xml")).unwrap();
        assert!(!sitemap.contains("draft"));
        assert!(!sitemap.contains("drafts"));

        fs::remove_dir_all(root).unwrap();
    }
}
