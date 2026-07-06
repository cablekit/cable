use crate::config::BlogConfig;
use crate::content::Post;
use crate::errors::BuildError;

struct PageMeta<'a> {
    title: &'a str,
    description: &'a str,
    canonical_url: Option<&'a str>,
    body_class: &'a str,
    og_type: &'a str,
}

pub fn render_index(config: &BlogConfig, posts: &[Post]) -> Result<String, BuildError> {
    let site_title = html_escape::encode_text(&config.site.title);
    let site_description = html_escape::encode_text(&config.site.description);

    let header = render_site_header(config);
    let post_list = render_post_list(posts);

    let body = format!(
        r#"<main class="site">
    {header}

    <section class="hero">
        <p class="eyebrow">Latest posts</p>
        <h1>{site_title}</h1>
        <p>{site_description}</p>
    </section>

    <section class="post-list" aria-label="Posts">
        {post_list}
    </section>
</main>"#
    );

    render_page(
        config,
        PageMeta {
            title: &config.site.title,
            description: &config.site.description,
            canonical_url: None,
            body_class: "page page-index",
            og_type: "website",
        },
        &body,
    )
}

pub fn render_post(config: &BlogConfig, post: &Post) -> Result<String, BuildError> {
    let page_title = format!("{} | {}", post.title, config.site.title);

    let post_title = html_escape::encode_text(&post.title);

    let date = &post.date.to_string();
    let post_date = html_escape::encode_text(date);
    let datetime = html_escape::encode_double_quoted_attribute(date);

    let header = render_site_header(config);
    let tags = render_tags(&post.tags);

    // post.html is already rendered Markdown HTML.
    let post_body = &post.html;

    let body = format!(
        r#"<main class="site">
    {header}

    <article class="post">
        <header class="post-header">
            <a class="back-link" href="/">&larr; All posts</a>

            <h1>{post_title}</h1>

            <div class="post-meta">
                <time datetime="{datetime}">{post_date}</time>
                {tags}
            </div>
        </header>

        <div class="post-content">
            {post_body}
        </div>
    </article>
</main>"#
    );

    let canonical_url = absolute_url(&config.site.url, &post.url);

    render_page(
        config,
        PageMeta {
            title: &page_title,
            description: &config.site.description,
            canonical_url: Some(&canonical_url),
            body_class: "page page-post",
            og_type: "article",
        },
        &body,
    )
}

fn render_page(
    config: &BlogConfig,
    meta: PageMeta<'_>,
    body: &str,
) -> Result<String, BuildError> {
    let site_title = html_escape::encode_double_quoted_attribute(&config.site.title);
    let page_title_text = html_escape::encode_text(meta.title);
    let page_title_attr = html_escape::encode_double_quoted_attribute(meta.title);
    let description = html_escape::encode_double_quoted_attribute(meta.description);
    let body_class = html_escape::encode_double_quoted_attribute(meta.body_class);
    let og_type = html_escape::encode_double_quoted_attribute(meta.og_type);

    let canonical = match meta.canonical_url {
        Some(url) => {
            let url = html_escape::encode_double_quoted_attribute(url);
            format!(r#"<link rel="canonical" href="{url}">"#)
        }
        None => String::new(),
    };

    let html = format!(
        r#"<!doctype html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{page_title_text}</title>
    <meta name="description" content="{description}">
    <meta name="generator" content="Cable">
    <meta property="og:title" content="{page_title_attr}">
    <meta property="og:description" content="{description}">
    <meta property="og:site_name" content="{site_title}">
    <meta property="og:type" content="{og_type}">
    {canonical}
    <link rel="stylesheet" href="/style.css">
</head>
<body class="{body_class}">
{body}
</body>
</html>"#
    );

    Ok(html)
}

fn render_site_header(config: &BlogConfig) -> String {
    let title = html_escape::encode_text(&config.site.title);
    let description = html_escape::encode_text(&config.site.description);

    format!(
        r#"<header class="site-header">
    <div>
        <a class="site-title" href="/">{title}</a>
        <p class="site-description">{description}</p>
    </div>
</header>"#
    )
}

fn render_post_list(posts: &[Post]) -> String {
    if posts.is_empty() {
        return r#"<p class="empty-state">No posts published yet.</p>"#.to_string();
    }

    posts
        .iter()
        .map(render_post_card)
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_post_card(post: &Post) -> String {
    let title = html_escape::encode_text(&post.title);

    let date = &post.date.to_string();
    let date_text = html_escape::encode_text(date);
    let datetime = html_escape::encode_double_quoted_attribute(date);
    let url = html_escape::encode_double_quoted_attribute(&post.url);
    let tags = render_tags(&post.tags);

    format!(
        r#"<article class="post-card">
    <div class="post-card-meta">
        <time datetime="{datetime}">{date_text}</time>
        {tags}
    </div>

    <h2 class="post-card-title">
        <a href="{url}">{title}</a>
    </h2>
</article>"#
    )
}

fn render_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        return String::new();
    }

    let tags = tags
        .iter()
        .map(|tag| {
            let label = html_escape::encode_text(tag);
            format!(r#"<span class="tag">{label}</span>"#)
        })
        .collect::<Vec<_>>()
        .join("");

    format!(r#"<div class="tags">{tags}</div>"#)
}

fn absolute_url(site_url: &str, path: &str) -> String {
    let site_url = site_url.trim_end_matches('/');
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };

    format!("{site_url}{path}")
}

pub fn default_css() -> &'static str {
    r#":root {
    color-scheme: light dark;

    --bg: #ffffff;
    --text: #171717;
    --muted: #666666;
    --border: #e5e5e5;
    --surface: #f7f7f7;
    --accent: #2563eb;
    --accent-soft: #dbeafe;

    font-family:
        Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont,
        "Segoe UI", sans-serif;
    line-height: 1.6;
}

@media (prefers-color-scheme: dark) {
    :root {
        --bg: #0a0a0a;
        --text: #f5f5f5;
        --muted: #a3a3a3;
        --border: #262626;
        --surface: #171717;
        --accent: #60a5fa;
        --accent-soft: #172554;
    }
}

* {
    box-sizing: border-box;
}

html {
    text-size-adjust: 100%;
}

body {
    margin: 0;
    background: var(--bg);
    color: var(--text);
}

a {
    color: inherit;
    text-decoration-color: color-mix(in srgb, currentColor 35%, transparent);
    text-decoration-thickness: 0.08em;
    text-underline-offset: 0.18em;
}

a:hover {
    color: var(--accent);
    text-decoration-color: currentColor;
}

:focus-visible {
    outline: 3px solid var(--accent);
    outline-offset: 3px;
}

.site {
    width: min(760px, calc(100% - 2rem));
    margin: 0 auto;
    padding: 3rem 0 5rem;
}

.site-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 4rem;
}

.site-title {
    font-weight: 700;
    text-decoration: none;
    letter-spacing: -0.03em;
}

.site-description {
    margin: 0.35rem 0 0;
    color: var(--muted);
    font-size: 0.95rem;
}

.hero {
    margin-bottom: 3rem;
    padding-bottom: 2.5rem;
    border-bottom: 1px solid var(--border);
}

.hero .eyebrow {
    margin: 0 0 0.75rem;
    color: var(--accent);
    font-size: 0.85rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
}

.hero h1 {
    margin: 0;
    font-size: clamp(2.5rem, 10vw, 5rem);
    line-height: 0.95;
    letter-spacing: -0.075em;
}

.hero p {
    max-width: 58ch;
    margin: 1rem 0 0;
    color: var(--muted);
    font-size: 1.1rem;
}

.post-list {
    display: grid;
    gap: 0;
}

.post-card {
    padding: 1.5rem 0;
    border-bottom: 1px solid var(--border);
}

.post-card:first-child {
    padding-top: 0;
}

.post-card-meta,
.post-meta {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.75rem;
    color: var(--muted);
    font-size: 0.9rem;
}

.post-card-title {
    margin: 0.5rem 0 0;
    font-size: clamp(1.35rem, 3vw, 1.8rem);
    line-height: 1.2;
    letter-spacing: -0.04em;
}

.post-card-title a {
    text-decoration: none;
}

.post-card-title a:hover {
    text-decoration: underline;
}

.tags {
    display: inline-flex;
    flex-wrap: wrap;
    gap: 0.4rem;
}

.tag {
    display: inline-flex;
    align-items: center;
    border-radius: 999px;
    padding: 0.15rem 0.55rem;
    background: var(--surface);
    color: var(--muted);
    font-size: 0.78rem;
    line-height: 1.4;
}

.back-link {
    display: inline-flex;
    margin-bottom: 2rem;
    color: var(--muted);
    font-size: 0.95rem;
    text-decoration: none;
}

.back-link:hover {
    color: var(--accent);
}

.post-header {
    margin-bottom: 3rem;
    padding-bottom: 2rem;
    border-bottom: 1px solid var(--border);
}

.post-header h1 {
    max-width: 11ch;
    margin: 0;
    font-size: clamp(2.5rem, 9vw, 5.5rem);
    line-height: 0.95;
    letter-spacing: -0.075em;
}

.post-header .post-meta {
    margin-top: 1.25rem;
}

.post-content {
    font-size: 1.075rem;
}

.post-content > * {
    margin-top: 0;
    margin-bottom: 1.25rem;
}

.post-content > * + h2,
.post-content > * + h3,
.post-content > * + h4 {
    margin-top: 2.5rem;
}

.post-content h1,
.post-content h2,
.post-content h3,
.post-content h4 {
    line-height: 1.2;
    letter-spacing: -0.035em;
}

.post-content h2 {
    font-size: 1.75rem;
}

.post-content h3 {
    font-size: 1.35rem;
}

.post-content p,
.post-content li {
    max-width: 68ch;
}

.post-content ul,
.post-content ol {
    padding-left: 1.5rem;
}

.post-content li + li {
    margin-top: 0.4rem;
}

.post-content blockquote {
    margin-left: 0;
    padding: 0.25rem 0 0.25rem 1.25rem;
    border-left: 3px solid var(--accent);
    color: var(--muted);
}

.post-content pre {
    max-width: 100%;
    overflow-x: auto;
    border: 1px solid var(--border);
    border-radius: 0.75rem;
    padding: 1rem;
    background: var(--surface);
}

.post-content code {
    font-family:
        ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas,
        "Liberation Mono", monospace;
    font-size: 0.92em;
}

.post-content :not(pre) > code {
    border-radius: 0.35rem;
    padding: 0.15rem 0.35rem;
    background: var(--surface);
}

.post-content img {
    max-width: 100%;
    height: auto;
    border-radius: 0.75rem;
}

.post-content hr {
    margin: 2.5rem 0;
    border: 0;
    border-top: 1px solid var(--border);
}

.empty-state {
    padding: 2rem;
    border: 1px dashed var(--border);
    border-radius: 0.75rem;
    color: var(--muted);
    background: var(--surface);
}

@media (max-width: 640px) {
    .site {
        padding-top: 1.5rem;
    }

    .site-header {
        display: block;
        margin-bottom: 3rem;
    }

    .post-header h1 {
        max-width: none;
    }
}
"#
}