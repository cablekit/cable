use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum BuildError {
    CheckPathExists {
        path: PathBuf,
        source: io::Error,
    },
    CleanDirectory {
        path: PathBuf,
        source: io::Error,
    },
    CreateDirectory {
        path: PathBuf,
        source: io::Error,
    },
    ReadDirectory {
        path: PathBuf,
        source: io::Error,
    },
    ReadDirectoryEntry {
        directory: PathBuf,
        source: io::Error,
    },
    ReadFile {
        path: PathBuf,
        source: io::Error,
    },
    WriteFile {
        path: PathBuf,
        source: io::Error,
    },
    CopyFile {
        source_path: PathBuf,
        destination_path: PathBuf,
        source: io::Error,
    },
    ParseConfig {
        path: PathBuf,
        source: toml::de::Error,
    },
    MissingOpeningFrontmatter {
        path: PathBuf,
    },
    MissingClosingFrontmatter {
        path: PathBuf,
    },
    ParseFrontmatter {
        path: PathBuf,
        source: serde_yaml::Error,
    },
    ParseDate {
        path: PathBuf,
        date: String,
        source: chrono::ParseError,
    },
    MissingConfigFile {
        path: PathBuf,
    },
    MissingPostsDirectory {
        path: PathBuf,
    },
    InvalidSlug {
        path: PathBuf,
        slug: String,
    },
    BrokenMarkdownLink {
        path: PathBuf,
        link: String,
        target: PathBuf,
    },
    MissingRouteSlug {
        route: String,
    },
    SourceOutsidePostsDirectory {
        posts_dir: PathBuf,
        source_path: PathBuf,
    },
    DuplicateSlug {
        slug: String,
        first_path: PathBuf,
        second_path: PathBuf,
    },
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildError::CheckPathExists { path, .. } => {
                write!(f, "could not check whether path exists: {}", path.display())
            }
            BuildError::CleanDirectory { path, .. } => {
                write!(f, "could not clean directory: {}", path.display())
            }
            BuildError::CreateDirectory { path, .. } => {
                write!(f, "could not create directory: {}", path.display())
            }
            BuildError::ReadDirectory { path, .. } => {
                write!(f, "could not read directory: {}", path.display())
            }
            BuildError::ReadDirectoryEntry { directory, .. } => {
                write!(
                    f,
                    "could not read an entry in directory: {}",
                    directory.display()
                )
            }
            BuildError::ReadFile { path, .. } => {
                write!(f, "could not read file: {}", path.display())
            }
            BuildError::WriteFile { path, .. } => {
                write!(f, "could not write file: {}", path.display())
            }
            BuildError::CopyFile {
                source_path,
                destination_path,
                ..
            } => write!(
                f,
                "could not copy file from {} to {}",
                source_path.display(),
                destination_path.display()
            ),
            BuildError::ParseConfig { path, .. } => {
                write!(f, "could not parse config file: {}", path.display())
            }
            BuildError::MissingOpeningFrontmatter { path } => {
                write!(
                    f,
                    "missing opening frontmatter delimiter in {}",
                    path.display()
                )
            }
            BuildError::MissingClosingFrontmatter { path } => {
                write!(
                    f,
                    "missing closing frontmatter delimiter in {}",
                    path.display()
                )
            }
            BuildError::ParseFrontmatter { path, .. } => {
                write!(f, "could not parse YAML frontmatter in {}", path.display())
            }
            BuildError::ParseDate { path, date, .. } => {
                write!(f, "could not parse date '{date}' in {}", path.display())
            }
            BuildError::MissingConfigFile { path } => {
                write!(f, "missing config file: {}", path.display())
            }
            BuildError::MissingPostsDirectory { path } => {
                write!(f, "missing posts directory: {}", path.display())
            }
            BuildError::InvalidSlug { path, slug } => {
                write!(f, "invalid slug '{slug}' in {}", path.display())
            }
            BuildError::BrokenMarkdownLink { path, link, target } => write!(
                f,
                "broken local Markdown link '{link}' in {} points to {}",
                path.display(),
                target.display()
            ),
            BuildError::MissingRouteSlug { route } => {
                write!(f, "post route must contain :slug: {route}")
            }
            BuildError::SourceOutsidePostsDirectory {
                posts_dir,
                source_path,
            } => write!(
                f,
                "post source path {} is not inside posts directory {}",
                source_path.display(),
                posts_dir.display()
            ),
            BuildError::DuplicateSlug {
                slug,
                first_path,
                second_path,
            } => write!(
                f,
                "duplicate slug '{slug}' found in {} and {}",
                first_path.display(),
                second_path.display()
            ),
        }
    }
}

impl Error for BuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            BuildError::CheckPathExists { source, .. }
            | BuildError::CleanDirectory { source, .. }
            | BuildError::CreateDirectory { source, .. }
            | BuildError::ReadDirectory { source, .. }
            | BuildError::ReadDirectoryEntry { source, .. }
            | BuildError::ReadFile { source, .. }
            | BuildError::WriteFile { source, .. }
            | BuildError::CopyFile { source, .. } => Some(source),
            BuildError::ParseConfig { source, .. } => Some(source),
            BuildError::ParseFrontmatter { source, .. } => Some(source),
            BuildError::ParseDate { source, .. } => Some(source),
            BuildError::MissingOpeningFrontmatter { .. }
            | BuildError::MissingClosingFrontmatter { .. }
            | BuildError::MissingConfigFile { .. }
            | BuildError::MissingPostsDirectory { .. }
            | BuildError::InvalidSlug { .. }
            | BuildError::BrokenMarkdownLink { .. }
            | BuildError::MissingRouteSlug { .. }
            | BuildError::SourceOutsidePostsDirectory { .. }
            | BuildError::DuplicateSlug { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn display_includes_context_for_duplicate_slug() {
        let error = BuildError::DuplicateSlug {
            slug: String::from("hello"),
            first_path: PathBuf::from("content/posts/hello.md"),
            second_path: PathBuf::from("content/posts/other.md"),
        };

        let message = error.to_string();

        assert!(message.contains("duplicate slug 'hello'"));
        assert!(message.contains("content/posts/hello.md"));
        assert!(message.contains("content/posts/other.md"));
    }

    #[test]
    fn source_returns_underlying_io_error() {
        let error = BuildError::ReadFile {
            path: PathBuf::from("missing.md"),
            source: io::Error::new(io::ErrorKind::NotFound, "missing"),
        };

        assert!(error.source().is_some());
    }
}
