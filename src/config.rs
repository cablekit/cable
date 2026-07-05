use crate::errors::BuildError;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct BlogConfig {
    pub site: SiteConfig,
    pub content: ContentConfig,
    pub output: OutputConfig,
    pub routes: RouteConfig,
}

impl BlogConfig {
    pub fn new(path: PathBuf) -> Result<BlogConfig, BuildError> {
        let toml_config = fs::read_to_string(&path).map_err(|source| BuildError::ReadFile {
            path: path.clone(),
            source,
        })?;
        let config: BlogConfig = toml::from_str(&toml_config)
            .map_err(|source| BuildError::ParseConfig { path, source })?;
        Ok(config)
    }
}

#[derive(Deserialize, Debug)]
pub struct SiteConfig {
    pub title: String,
    pub description: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct ContentConfig {
    pub posts: String,
}

#[derive(Deserialize, Debug)]
pub struct OutputConfig {
    pub directory: PathBuf,
}

#[derive(Deserialize, Debug)]
pub struct RouteConfig {
    pub post: String,
}

pub fn load_config(config_path: PathBuf) -> Result<BlogConfig, BuildError> {
    let config = BlogConfig::new(config_path)?;
    Ok(config)
}
