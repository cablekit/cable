use crate::errors::BuildError;
use pulldown_cmark::{Options, Parser, html};

pub fn to_html(md_body: &str) -> Result<String, BuildError> {
    let mut options = Options::empty();

    // Reasonable blog defaults.
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(md_body, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    Ok(html_output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_headers_and_paragraphs_to_html() {
        let html = to_html("# Hello\n\nThis is a paragraph.").unwrap();

        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("<p>This is a paragraph.</p>"));
    }
}
