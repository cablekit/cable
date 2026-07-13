# Cable

Cable is a static blog generator written in Rust.

It builds a blog from Markdown files and a TOML config. The output is
plain HTML and CSS, so it can be hosted anywhere.

## Getting Started

Build the example blog:

```powershell
cargo run build --root examples/basic
```

The generated site will be written to `examples/basic/dist`.

To run it locally:

```powershell
cargo run dev --root examples/basic
```

Open `http://127.0.0.1:3119`.

## Commands

Build a site:

```powershell
cargo run build --root path/to/site
```

Validate a site:

```powershell
cargo run validate --root path/to/site
```

Start the dev server:

```powershell
cargo run dev --root path/to/site --port 3119
```

Create a post:

```powershell
cargo run new post "My Post Title" --root path/to/site
```

## Site Setup

A Cable site needs a `blog.toml` file and a posts directory.

```text
my-blog/
  blog.toml
  content/
    posts/
      hello-world.md
  public/
  dist/
```

Example config:

```toml
[site]
title = "My Blog"
description = "Notes and writing"
url = "https://example.com"

[content]
posts = "content/posts"

[output]
directory = "dist"

[routes]
post = "/posts/:slug"
```

## Writing Posts

Posts are Markdown files with front matter.

```markdown
---
title: "Hello World"
date: "2026-06-27"
slug: "hello-world"
tags:
  - intro
status: "published"
---

# Hello World

This is my first post.
```

Set `status` to `draft` to keep a post out of the build.

## Development

Run the tests:

```powershell
cargo test
```

Check formatting:

```powershell
cargo fmt --check
```

## Current Development Roadmap
### Phase 2 v0.2.0
- ~~Dev Server~~
- ~~Post Command~~
- ~~Validate Command~~
- Init Command
- Robots.txt
- RSS Feed
- Better Content Model
- Tag Pages
- Archive Page
- Sitemap
- Layout Updates
  - site header
  - navigation
  - footer
  - tag links
  - archive link
  - feed link
  - better typography
  - responsive layout
  - basic SEO
- Better Errors
- Internal Link Checking?

## Further Reading
To read my progress on CABLE please find my blog here https://thecassiofeed.com 