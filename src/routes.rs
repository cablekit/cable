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

pub fn post_output_path(output_dir: &Path, url: &str) -> PathBuf {
    output_dir.join(url.trim_start_matches('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_url_uses_nested_directory_from_source_path() {
        let posts_dir = PathBuf::from("content/posts");
        let source_path = posts_dir.join("reviews").join("nested.md");

        let url = post_url("/posts/:slug", "nested", &posts_dir, &source_path).unwrap();

        assert_eq!(url, "/posts/reviews/nested.html");
    }

    #[test]
    fn post_url_rejects_routes_without_slug_placeholder() {
        let posts_dir = PathBuf::from("content/posts");
        let source_path = posts_dir.join("hello.md");

        let error = post_url("/posts", "hello", &posts_dir, &source_path).unwrap_err();

        assert!(matches!(error, BuildError::MissingRouteSlug { .. }));
    }

    #[test]
    fn post_output_path_joins_url_under_output_directory() {
        let output_dir = PathBuf::from("dist");
        let url = String::from("/posts/hello.html");

        let output_path = post_output_path(&output_dir, &url);

        assert_eq!(output_path, PathBuf::from("dist").join("posts/hello.html"));
    }
}
