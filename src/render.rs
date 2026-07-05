use crate::config::BlogConfig;
use crate::content::Post;
use crate::errors::BuildError;

pub fn render_index(config: &BlogConfig, posts: &[Post]) -> Result<String, BuildError> {
    let site_title = html_escape::encode_text(&config.site.title);
    let site_description = html_escape::encode_text(&config.site.description);

    let mut post_items = String::new();

    for post in posts {
        let title = html_escape::encode_text(&post.title);
        let date_string = &post.date.to_string();
        let date = html_escape::encode_text(date_string);
        let url = html_escape::encode_double_quoted_attribute(&post.url);

        let tags = if post.tags.is_empty() {
            String::new()
        } else {
            let tag_list = post
                .tags
                .iter()
                .map(|tag| {
                    let tag = html_escape::encode_text(tag);
                    format!(r#"<span class="tag">{tag}</span>"#)
                })
                .collect::<Vec<_>>()
                .join("");

            format!(r#"<div class="post-tags">{tag_list}</div>"#)
        };

        post_items.push_str(&format!(
            r#"
            <article class="post-card">
                <h2 class="post-title">
                    <a href="{url}">{title}</a>
                </h2>
                <time class="post-date">{date}</time>
                {tags}
            </article>
            "#
        ));
    }

    if post_items.is_empty() {
        post_items.push_str(
            r#"
            <p class="empty-state">No posts published yet.</p>
            "#,
        );
    }

    let html = format!(
        r#"<!doctype html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{site_title}</title>
    <meta name="description" content="{site_description}">
    <style>
        :root {{
            color-scheme: light dark;
            font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
            line-height: 1.5;
        }}

        body {{
            margin: 0;
            background: Canvas;
            color: CanvasText;
        }}

        .site {{
            width: min(720px, calc(100% - 2rem));
            margin: 0 auto;
            padding: 4rem 0;
        }}

        .site-header {{
            margin-bottom: 3rem;
        }}

        .site-title {{
            margin: 0;
            font-size: clamp(2rem, 8vw, 4rem);
            line-height: 1;
            letter-spacing: -0.05em;
        }}

        .site-description {{
            margin: 1rem 0 0;
            font-size: 1.125rem;
            color: color-mix(in srgb, CanvasText 70%, Canvas);
        }}

        .post-list {{
            display: grid;
            gap: 1.5rem;
        }}

        .post-card {{
            padding-top: 1.5rem;
            border-top: 1px solid color-mix(in srgb, CanvasText 20%, Canvas);
        }}

        .post-title {{
            margin: 0;
            font-size: 1.5rem;
            line-height: 1.2;
        }}

        .post-title a {{
            color: inherit;
            text-decoration-thickness: 0.08em;
            text-underline-offset: 0.2em;
        }}

        .post-title a:hover {{
            text-decoration-thickness: 0.14em;
        }}

        .post-date {{
            display: block;
            margin-top: 0.5rem;
            font-size: 0.95rem;
            color: color-mix(in srgb, CanvasText 65%, Canvas);
        }}

        .post-tags {{
            display: flex;
            flex-wrap: wrap;
            gap: 0.4rem;
            margin-top: 0.75rem;
        }}

        .tag {{
            display: inline-flex;
            border: 1px solid color-mix(in srgb, CanvasText 20%, Canvas);
            border-radius: 999px;
            padding: 0.15rem 0.55rem;
            font-size: 0.8rem;
            color: color-mix(in srgb, CanvasText 75%, Canvas);
        }}

        .empty-state {{
            padding-top: 1.5rem;
            border-top: 1px solid color-mix(in srgb, CanvasText 20%, Canvas);
            color: color-mix(in srgb, CanvasText 70%, Canvas);
        }}
    </style>
</head>
<body>
    <main class="site">
        <header class="site-header">
            <h1 class="site-title">{site_title}</h1>
            <p class="site-description">{site_description}</p>
        </header>

        <section class="post-list" aria-label="Posts">
            {post_items}
        </section>
    </main>
</body>
</html>"#
    );

    Ok(html)
}

pub fn render_post(config: &BlogConfig, post: &Post) -> Result<String, BuildError> {
    let site_title = html_escape::encode_text(&config.site.title);
    let site_description = html_escape::encode_text(&config.site.description);

    let post_title = html_escape::encode_text(&post.title);
    let date_string = &post.date.to_string();
    let post_date = html_escape::encode_text(date_string);

    let post_body = &post.html;

    let page_title = format!("{post_title} | {site_title}");
    let page_description = if site_description.is_empty() {
        post_title.to_string()
    } else {
        site_description.to_string()
    };

    let tags = if post.tags.is_empty() {
        String::new()
    } else {
        let tag_list = post
            .tags
            .iter()
            .map(|tag| {
                let tag = html_escape::encode_text(tag);
                format!(r#"<span class="tag">{tag}</span>"#)
            })
            .collect::<Vec<_>>()
            .join("");

        format!(r#"<div class="post-tags">{tag_list}</div>"#)
    };

    let html = format!(
        r#"<!doctype html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{page_title}</title>
    <meta name="description" content="{page_description}">
    <style>
        :root {{
            color-scheme: light dark;
            font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
            line-height: 1.6;
        }}

        body {{
            margin: 0;
            background: Canvas;
            color: CanvasText;
        }}

        .site {{
            width: min(720px, calc(100% - 2rem));
            margin: 0 auto;
            padding: 4rem 0;
        }}

        .site-header {{
            margin-bottom: 3rem;
        }}

        .home-link {{
            color: inherit;
            text-decoration-thickness: 0.08em;
            text-underline-offset: 0.2em;
        }}

        .home-link:hover {{
            text-decoration-thickness: 0.14em;
        }}

        .post-header {{
            margin-bottom: 3rem;
        }}

        .post-title {{
            margin: 0;
            font-size: clamp(2rem, 8vw, 4rem);
            line-height: 1;
            letter-spacing: -0.05em;
        }}

        .post-date {{
            display: block;
            margin-top: 1rem;
            font-size: 0.95rem;
            color: color-mix(in srgb, CanvasText 65%, Canvas);
        }}

        .post-tags {{
            display: flex;
            flex-wrap: wrap;
            gap: 0.4rem;
            margin-top: 1rem;
        }}

        .tag {{
            display: inline-flex;
            border: 1px solid color-mix(in srgb, CanvasText 20%, Canvas);
            border-radius: 999px;
            padding: 0.15rem 0.55rem;
            font-size: 0.8rem;
            color: color-mix(in srgb, CanvasText 75%, Canvas);
        }}

        .post-content {{
            font-size: 1.05rem;
        }}

        .post-content > * + * {{
            margin-top: 1.25rem;
        }}

        .post-content h1,
        .post-content h2,
        .post-content h3,
        .post-content h4 {{
            line-height: 1.2;
            margin-top: 2rem;
        }}

        .post-content h1 {{
            font-size: 2rem;
        }}

        .post-content h2 {{
            font-size: 1.6rem;
        }}

        .post-content h3 {{
            font-size: 1.3rem;
        }}

        .post-content a {{
            color: inherit;
            text-decoration-thickness: 0.08em;
            text-underline-offset: 0.2em;
        }}

        .post-content a:hover {{
            text-decoration-thickness: 0.14em;
        }}

        .post-content pre {{
            overflow-x: auto;
            padding: 1rem;
            border-radius: 0.5rem;
            background: color-mix(in srgb, CanvasText 8%, Canvas);
        }}

        .post-content code {{
            font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace;
            font-size: 0.95em;
        }}

        .post-content :not(pre) > code {{
            padding: 0.1rem 0.25rem;
            border-radius: 0.25rem;
            background: color-mix(in srgb, CanvasText 8%, Canvas);
        }}

        .post-content blockquote {{
            margin-left: 0;
            padding-left: 1rem;
            border-left: 4px solid color-mix(in srgb, CanvasText 25%, Canvas);
            color: color-mix(in srgb, CanvasText 75%, Canvas);
        }}

        .post-content img {{
            max-width: 100%;
            height: auto;
        }}

        .post-footer {{
            margin-top: 4rem;
            padding-top: 2rem;
            border-top: 1px solid color-mix(in srgb, CanvasText 20%, Canvas);
        }}
    </style>
</head>
<body>
    <main class="site">
        <header class="site-header">
            <a class="home-link" href="/">&larr; {site_title}</a>
        </header>

        <article class="post">
            <header class="post-header">
                <h1 class="post-title">{post_title}</h1>
                <time class="post-date">{post_date}</time>
                {tags}
            </header>

            <section class="post-content">
                {post_body}
            </section>
        </article>

        <footer class="post-footer">
            <a class="home-link" href="/">&larr; Back to all posts</a>
        </footer>
    </main>
</body>
</html>"#
    );

    Ok(html)
}
