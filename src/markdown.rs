//! Markdown rendering.
//!
//! Uses the [`pulldown_cmark`] crate with Github-flavored markdown options enabled. Extracts
//! languages used in code blocks for highlighting purposes.

use pulldown_cmark::{Event, Options, Parser, html};
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

// pub struct Renderer {
//     /// The original path given to the renderer.
//     pub path: PathBuf,
//
//     /// The canonicalized path to use in file operations.
//     pub canonical_path: PathBuf,
// }

/// Encapsulates a path and provides an interface to turn its contents into HTML.
///
pub trait Renderer {
    /// Create a new renderer instance that wraps the given file.
    ///
    fn new(path: PathBuf) -> Self;

    /// Turn the current contents of the file into HTML.
    ///
    fn run(&self) -> Result<RenderedContent, io::Error>;

    /// Gets the path of the file that's being rendered
    ///
    fn get_path(&self) -> PathBuf;

    /// Gets the canonical path of the file that's being rendered
    ///
    fn get_canonical_path(&self) -> PathBuf;
}

/// Encapsulates a markdown file and provides an interface to turn its contents into HTML.
///
pub struct MarkdownRenderer {
    ///Path to the file
    ///
    pub path: PathBuf,
    ///Full path to the file
    ///
    pub canonical_path: PathBuf,
}

impl Renderer for MarkdownRenderer {
    fn new(md_path: PathBuf) -> Self {
        let canonical_md_path = md_path.canonicalize().unwrap_or_else(|_| md_path.clone());

        MarkdownRenderer {
            path: md_path,
            canonical_path: canonical_md_path,
        }
    }

    fn get_path(&self) -> PathBuf {
        return self.path.clone();
    }

    fn get_canonical_path(&self) -> PathBuf {
        return self.canonical_path.clone();
    }

    fn run(&self) -> Result<RenderedContent, io::Error> {
        let markdown = fs::read_to_string(&self.canonical_path)?;
        let root_dir = self
            .canonical_path
            .parent()
            .unwrap_or_else(|| Path::new(""));

        let re_absolute_url = Regex::new(r"^[a-z]+://").unwrap();
        let re_path_prefix = Regex::new(r"^(/|\./)?").unwrap();

        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = Parser::new_ext(&markdown, options);

        let mut languages = HashSet::new();
        let parser = parser.map(|mut event| {
            use pulldown_cmark::{CodeBlockKind, Tag};

            match &mut event {
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(content))) => {
                    if content.len() > 0 {
                        languages.insert(content.to_string());
                    }
                }
                Event::Start(Tag::Image(_, url, _)) if !re_absolute_url.is_match(url) => {
                    *url = format!(
                        "file://{}/{}",
                        root_dir.display(),
                        re_path_prefix.replace(url, "")
                    )
                    .into();
                }
                _ => (),
            }

            event
        });

        let mut output = String::new();
        html::push_html(&mut output, parser);

        Ok(RenderedContent {
            html: output,
            code_languages: languages,
        })
    }
}

/// The output of the rendering process. Includes both the rendered HTML and additional metadata
/// used by its clients.
///
#[derive(Debug, Default)]
pub struct RenderedContent {
    /// The rendered HTML.
    pub html: String,

    /// All the languages in fenced code blocks from the markdown input.
    pub code_languages: HashSet<String>,
}
