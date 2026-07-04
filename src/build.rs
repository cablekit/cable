use std::error::Error;
use std::path::{PathBuf};

use crate::{config, fs};

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

    println!("Assets copied {}", copied_assets);

    Ok(BuildResult {
        output_dir: root.join("dist"),
        posts: 0,
        drafts: 0,
        copied_assets
    })
}


