use crate::config::BlogConfig;
use crate::content::Post;
use crate::render;

pub fn generate_feed(config: &BlogConfig, posts: &Vec<Post>) -> String {
    let site_title = &config.site.title;
    let site_url = &config.site.url;
    let site_description = &config.site.description;
    let post_xml_list = render_post_xml_list(site_url, posts);

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0">

<channel>
  <title>{site_title}</title>
  <link>{site_url}</link>
  <description>{site_description}</description>
  {post_xml_list}
</channel>

</rss>"#
    )
}

fn render_post_xml_list(site_url: &str, posts: &[Post]) -> String {
    if posts.is_empty() {
        return r#""#.to_string();
    }

    posts
        .iter()
        .map(|post| render_post_xml(site_url, post))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_post_xml(site_url: &str, post: &Post) -> String {
    let title = html_escape::encode_text(&post.title);
    let absolute_url = render::absolute_url(site_url, &post.url);
    let url = html_escape::encode_double_quoted_attribute(&absolute_url);

    format!(
        r#"
  <item>
    <title>{title}</title>
    <link>{url}</link>
    <description>New XML tutorial on W3Schools</description>
  </item>"#
    )
}
