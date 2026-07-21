use crate::content::{Post, title_to_slug};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Tag<'a> {
    pub name: String,
    pub slug: String,
    pub posts: Vec<&'a Post>,
}

pub fn generate_tags(posts: &[Post]) -> Vec<Tag<'_>> {
    let mut tags_by_name: BTreeMap<String, Vec<&Post>> = BTreeMap::new();

    for post in posts {
        for tag in &post.tags {
            tags_by_name.entry(tag.clone()).or_default().push(post);
        }
    }

    tags_by_name
        .into_iter()
        .map(|(name, posts)| Tag {
            slug: tag_to_slug(&name),
            name,
            posts,
        })
        .collect::<Vec<_>>()
}
pub fn tag_url(name: &str) -> String {
    let mut url = tag_to_slug(name);
    url.push_str(".html");
    url
}

fn tag_to_slug(name: &str) -> String {
    title_to_slug(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::Status;
    use chrono::NaiveDate;
    use std::path::PathBuf;

    fn post(title: &str, slug: &str, tags: Vec<&str>) -> Post {
        Post {
            title: title.to_string(),
            date: NaiveDate::from_ymd_opt(2026, 7, 21).unwrap(),
            slug: slug.to_string(),
            tags: tags.into_iter().map(String::from).collect(),
            status: Status::Published,
            body: String::new(),
            html: String::new(),
            source_path: PathBuf::from(format!("content/posts/{slug}.md")),
            output_path: PathBuf::from(format!("dist/posts/{slug}.html")),
            url: format!("/posts/{slug}.html"),
        }
    }

    #[test]
    fn generate_tags_groups_posts_by_tag_name() {
        let posts = vec![
            post("One", "one", vec!["rust", "static sites"]),
            post("Two", "two", vec!["rust"]),
        ];

        let tags = generate_tags(&posts);

        let rust = tags.iter().find(|tag| tag.name == "rust").unwrap();
        let static_sites = tags.iter().find(|tag| tag.name == "static sites").unwrap();

        assert_eq!(rust.slug, "rust");
        assert_eq!(rust.posts.len(), 2);
        assert_eq!(rust.posts[0].slug, "one");
        assert_eq!(rust.posts[1].slug, "two");
        assert_eq!(static_sites.slug, "static-sites");
        assert_eq!(static_sites.posts.len(), 1);
        assert_eq!(static_sites.posts[0].slug, "one");
    }

    #[test]
    fn tag_url_returns_slugged_html_filename() {
        assert_eq!(tag_url("Rust Tips"), "rust-tips.html");
    }
}
