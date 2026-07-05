use std::error::Error;
use std::fs;
use std::path::{PathBuf};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct BlogConfig{
    pub  site: SiteConfig,
    pub content: ContentConfig,
    pub  output: OutputConfig,
    pub  routes: RouteConfig
}

impl BlogConfig {
    pub fn new(path: PathBuf) -> Result<BlogConfig, Box<dyn Error>>{
        let toml_config = fs::read_to_string(path)?;
        let config: BlogConfig = toml::from_str(&toml_config)?;
        Ok(config)
    }
}

#[derive(Deserialize, Debug)]
pub struct  SiteConfig{
    pub  title: String,
    pub  description: String,
    pub  url: String
}

#[derive(Deserialize, Debug)]
pub struct ContentConfig{
    pub posts: String,
}

#[derive(Deserialize, Debug)]
pub struct OutputConfig {
    pub  directory: PathBuf
}

#[derive(Deserialize, Debug)]
pub struct RouteConfig{
    pub  post: String
}

pub fn load_config(config_path: PathBuf) -> Result<BlogConfig, Box<dyn Error>>{
    let config = BlogConfig::new(config_path)?;
    Ok(config)
}