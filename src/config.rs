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
    #[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_dir(name: &str) -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("cable-{name}-{id}"))
    }

    #[test]
    fn loads_blog_config_from_toml() {
        let root = test_dir("config");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("blog.toml");
        fs::write(
            &path,
            r#"
[site]
title = "Test Blog"
description = "A test"
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

        let config = load_config(path).unwrap();

        assert_eq!(config.site.title, "Test Blog");
        assert_eq!(config.content.posts, "content/posts");
        assert_eq!(config.output.directory, PathBuf::from("dist"));
        assert_eq!(config.routes.post, "/posts/:slug");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn invalid_toml_returns_parse_config_error() {
        let root = test_dir("bad-config");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("blog.toml");
        fs::write(&path, "not valid toml =").unwrap();

        let error = load_config(path).unwrap_err();

        assert!(matches!(error, BuildError::ParseConfig { .. }));
        fs::remove_dir_all(root).unwrap();
    }
}
