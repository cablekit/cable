use std::error::Error;
use std::path::PathBuf;

pub fn post_url(route: &String, slug: &String) -> Result<String, Box<dyn Error>> {
    if !route.contains(":slug") {
        return Err("post route must contain :slug".into());
    }

    let slug = slug.trim_matches('/');
    let mut url = route.replace(":slug", slug);

    url.push_str(".html");

    Ok(url)
}

pub fn post_output_path(output_dir: &PathBuf, url: &String) -> PathBuf {
    let output_path = output_dir.join(url.trim_start_matches('/'));
    output_path
}