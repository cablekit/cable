use std::collections::HashMap;
use std::path::PathBuf;

use crate::content::{Post, Status};
use crate::errors::BuildError;
use crate::{config, content, fs, markdown, render, routes};

#[derive(Debug)]
pub struct BuildResult {
    output_dir: PathBuf,
    posts: usize,
    drafts: usize,
    copied_assets: usize,
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

    validate_duplicate_slugs(&*posts)?;

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

    published_posts.sort_by(|a, b| b.date.cmp(&a.date));

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
        drafts: drafts,
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
