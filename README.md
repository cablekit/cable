# C.A.B.L.E.
## Cass's Automatic Blogging & Layout Engine



This is the cable project. 

## Phase 1 Goal

Build a Rust CLI that can take this:

```txt
my-blog/
  blog.toml
  content/
    posts/
      hello-world.md
  public/
    logo.svg
```

And generate this:

```txt
my-blog/dist/
  index.html
  posts/
    hello-world/
      index.html
  logo.svg
```

The core promise for Phase 1 is:

```txt
A user writes content and config as files.
The Rust CLI reads those files.
The Rust CLI generates a static blog into dist/.
The user can host dist/ anywhere.
```

## Phase 1 Includes

Phase 1 includes:

```txt
- Rust CLI project
- build command
- blog.toml config file
- Markdown post discovery
- YAML frontmatter parsing
- Markdown to HTML conversion
- Static index page generation
- Static post page generation
- public/ asset copying
- draft skipping
- duplicate slug detection
- readable build summary
- readable error messages for common failures
```

---

## Phase 1 Does Not Include

Phase 1 intentionally does not include:

```txt
- admin dashboard
- database
- authentication
- comments
- plugin system
- theme system
- RSS
- sitemap
- search
- MDX
- image optimization
- dev server
- hot reload
- deployment adapters
- importers
- analytics
- visual editor
```

