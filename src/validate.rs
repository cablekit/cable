use std::collections::HashMap;
use std::path::{Path, PathBuf};

use pulldown_cmark::{Event, LinkType, Parser, Tag};

use crate::config::BlogConfig;
use crate::content::{Post, Status};
use crate::errors::BuildError;
use crate::{config, content, routes};

#[derive(Debug)]
pub struct ValidateResult {
    pub posts: usize,
    pub drafts: usize,
}

pub(crate) struct ValidatedSite {
    pub config: BlogConfig,
    pub posts_dir: PathBuf,
    pub output_dir: PathBuf,
    pub posts: Vec<Post>,
    pub drafts: usize,
}

pub fn validate_build(root: PathBuf) -> Result<ValidateResult, BuildError> {
    println!("Validating site at: {}", root.display());

    let site = validate_site(root)?;
    let posts = site
        .posts
        .iter()
        .filter(|post| post.status == Status::Published)
        .count();

    Ok(ValidateResult {
        posts,
        drafts: site.drafts,
    })
}

pub(crate) fn validate_site(root: PathBuf) -> Result<ValidatedSite, BuildError> {
    let config_path = root.join("blog.toml");
    if !config_path.is_file() {
        return Err(BuildError::MissingConfigFile { path: config_path });
    }

    let config = config::load_config(config_path)?;
    let posts_dir = root.join(&config.content.posts);
    if !posts_dir.is_dir() {
        return Err(BuildError::MissingPostsDirectory { path: posts_dir });
    }

    let output_dir = root.join(&config.output.directory);
    let source_files = content::discover_markdown_files(&posts_dir)?;
    let mut posts: Vec<Post> = Vec::new();

    for source_path in source_files {
        let post = content::post_from_path_location(&source_path)?;

        validate_slug(&post)?;
        validate_local_markdown_links(&post, &source_path)?;

        posts.push(post);
    }

    validate_duplicate_slugs(&posts)?;

    for post in &posts {
        routes::post_url(
            &config.routes.post,
            &post.slug,
            &posts_dir,
            &post.source_path,
        )?;
    }

    let drafts = posts
        .iter()
        .filter(|post| post.status == Status::Draft)
        .count();

    Ok(ValidatedSite {
        config,
        posts_dir,
        output_dir,
        posts,
        drafts,
    })
}

fn validate_slug(post: &Post) -> Result<(), BuildError> {
    let valid = !post.slug.is_empty()
        && post
            .slug
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
        && !post.slug.starts_with('-')
        && !post.slug.ends_with('-')
        && !post.slug.contains("--");

    if valid {
        Ok(())
    } else {
        Err(BuildError::InvalidSlug {
            path: post.source_path.clone(),
            slug: post.slug.clone(),
        })
    }
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

fn validate_local_markdown_links(post: &Post, source_path: &Path) -> Result<(), BuildError> {
    let source_dir = source_path.parent().unwrap_or_else(|| Path::new(""));

    for event in Parser::new(&post.body) {
        if let Event::Start(Tag::Link {
            link_type,
            dest_url,
            ..
        }) = event
        {
            if !matches!(
                link_type,
                LinkType::Inline | LinkType::Reference | LinkType::Collapsed | LinkType::Shortcut
            ) {
                continue;
            }

            let link = dest_url.as_ref();
            if should_skip_link(link) {
                continue;
            }

            let target_path = strip_fragment(link);
            let target = source_dir.join(target_path);
            if !target.is_file() {
                return Err(BuildError::BrokenMarkdownLink {
                    path: source_path.to_path_buf(),
                    link: link.to_string(),
                    target,
                });
            }
        }
    }

    Ok(())
}

fn should_skip_link(link: &str) -> bool {
    link.is_empty()
        || link.starts_with('#')
        || link.starts_with('/')
        || link.contains("://")
        || link.starts_with("mailto:")
        || link.starts_with("tel:")
}

fn strip_fragment(link: &str) -> &str {
    link.split(['#', '?']).next().unwrap_or(link)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

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
    fn validate_slug_rejects_invalid_slug() {
        let post = post("Hello World", PathBuf::from("content/posts/hello.md"));

        let error = validate_slug(&post).unwrap_err();

        assert!(matches!(error, BuildError::InvalidSlug { .. }));
    }
}
