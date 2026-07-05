use std::error::Error;
use pulldown_cmark::{html, Options, Parser};

pub fn to_html(md_body: &str) -> Result<String, Box<dyn Error>>{


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