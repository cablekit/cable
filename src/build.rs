use std::collections::HashMap;
use std::error::Error;
use std::path::{PathBuf};

use crate::{config, content, fs, markdown, routes};
use crate::content::{Post, Status};

#[derive(Debug)]
pub struct BuildResult {
    output_dir: PathBuf,
    posts: usize,
    drafts: usize,
    copied_assets: usize
}



pub fn build_site(root: PathBuf) -> Result<BuildResult, Box<dyn Error>>{
    println!("Building site at: {:?}", root);

    let config_path = root.join("blog.toml");
    let config = config::load_config(config_path)?;

    let posts_dir = root.join(&config.content.posts);
    let output_dir = root.join(&config.output.directory);
    let public_dir = root.join("public");


    fs::clean_dir(&output_dir)?;
    fs::ensure_dir(&output_dir)?;


    //Non-recursive copy of assets. This skips over any directories
    let copied_assets = if public_dir.exists(){
        fs::copy_dir_contents(&public_dir, &output_dir)?
    } else{
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
        )?;

        post.output_path = routes::post_output_path(
            &output_dir,
            &post.url
        );
    }

    let drafts = posts
        .iter()
        .filter(|post| post.status == Status::Draft)
        .count();

    let mut published_posts = posts
        .into_iter()
        .filter(|post| post.status == Status::Published)
        .collect::<Vec<_>>();

    published_posts.sort_by(|a, b| {
        b.date.cmp(&a.date)
    });

    println!("Published Posts {:#?}", published_posts);

    Ok(BuildResult {
        output_dir: root.join("dist"),
        posts: 0,
        drafts: drafts,
        copied_assets
    })
}

fn validate_duplicate_slugs(posts: &[Post]) -> Result<(), Box<dyn Error>> {
    let mut seen = HashMap::new();

    for post in posts {
        if let Some(previous_path) = seen.get(&post.slug) {
            return Err(Box::from("Duplicate Slug"))
            // return Err(BuildError::DuplicateSlug {
            //     slug: post.slug.clone(),
            //     first_path: previous_path.clone(),
            //     second_path: post.source_path.clone(),
            // });
        }

        seen.insert(
            post.slug.clone(),
            post.source_path.clone(),
        );
    }

    Ok(())
}


