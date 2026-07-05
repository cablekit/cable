use crate::errors::BuildError;
use std::path::{Path, PathBuf};

pub fn post_url(
    route: &str,
    slug: &str,
    posts_dir: &Path,
    source_path: &Path,
) -> Result<String, BuildError> {
    if !route.contains(":slug") {
        return Err(BuildError::MissingRouteSlug {
            route: route.to_string(),
        });
    }

    let relative_path = source_path.strip_prefix(posts_dir).map_err(|_| {
        BuildError::SourceOutsidePostsDirectory {
            posts_dir: posts_dir.to_path_buf(),
            source_path: source_path.to_path_buf(),
        }
    })?;
    let nested_dir = relative_path.parent().unwrap_or(Path::new(""));

    let slug = slug.trim_matches('/');

    let slug_path = if nested_dir.as_os_str().is_empty() {
        slug.to_string()
    } else {
        format!(
            "{}/{}",
            nested_dir.to_string_lossy().replace('\\', "/"),
            slug
        )
    };
    let mut url = route.replace(":slug", &slug_path);

    url.push_str(".html");

    Ok(url)
}

pub fn post_output_path(output_dir: &PathBuf, url: &String) -> PathBuf {
    let output_path = output_dir.join(url.trim_start_matches('/'));
    output_path
}
