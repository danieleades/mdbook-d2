use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::bail;
use mdbook::book::SectionNumber;
use mdbook::preprocess::PreprocessorContext;
use pulldown_cmark::{CowStr, Event, LinkType, Tag, TagEnd};

use crate::config::Config;

/// Represents the backend for processing D2 diagrams
pub struct Backend {
    /// Absolute path to the D2 binary
    path: PathBuf,
    /// Relative path to the output directory for generated diagrams
    output_dir: PathBuf,
    /// Absolute path to the source directory of the book
    source_dir: PathBuf,
    /// Layout engine to use for D2 diagrams
    layout: String,
}

/// Context for rendering a specific diagram
#[derive(Debug, Clone, Copy)]
pub struct RenderContext<'a> {
    /// Relative path to the current chapter file
    path: &'a Path,
    /// Name of the current chapter
    chapter: &'a str,
    /// Section number of the current chapter
    section: Option<&'a SectionNumber>,
    /// Index of the current diagram within the chapter
    diagram_index: usize,
}

impl<'a> RenderContext<'a> {
    /// Creates a new [`RenderContext`]
    pub const fn new(
        path: &'a Path,
        chapter: &'a str,
        section: Option<&'a SectionNumber>,
        diagram_index: usize,
    ) -> Self {
        Self {
            path,
            chapter,
            section,
            diagram_index,
        }
    }
}

/// Generates a filename for a diagram based on its context
///
/// Returns a relative path for the diagram file
fn filename(ctx: &RenderContext) -> String {
    format!(
        "{}{}.svg",
        ctx.section.cloned().unwrap_or_default(),
        ctx.diagram_index
    )
}

impl Backend {
    /// Creates a new Backend instance
    ///
    /// # Arguments
    /// * `config` - Configuration for the D2 preprocessor
    /// * `source_dir` - Absolute path to the book's source directory
    pub fn new(config: Config, source_dir: PathBuf) -> Self {
        Self {
            path: config.path,
            output_dir: config.output_dir,
            layout: config.layout,
            source_dir,
        }
    }

    /// Creates a Backend instance from a [`PreprocessorContext`]
    ///
    /// # Arguments
    /// * `ctx` - The preprocessor context
    pub fn from_context(ctx: &PreprocessorContext) -> Self {
        let toml_value: toml::Value = ctx
            .config
            .get_preprocessor("d2")
            .expect("d2 preprocessor config not found")
            .clone()
            .into();
        let config: Config = toml_value.try_into().expect("cannot convert toml config");
        let source_dir = ctx.root.join(&ctx.config.book.src);

        Self::new(config, source_dir)
    }

    /// Returns the relative path to the output directory
    fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    /// Constructs the absolute file path for a diagram
    ///
    /// # Arguments
    /// * `ctx` - The render context for the diagram
    fn filepath(&self, ctx: &RenderContext) -> PathBuf {
        let filepath = Path::new(&self.source_dir).join(self.relative_file_path(ctx));
        filepath
    }

    /// Constructs the relative file path for a diagram
    ///
    /// # Arguments
    /// * `ctx` - The render context for the diagram
    fn relative_file_path(&self, ctx: &RenderContext) -> PathBuf {
        let filename = filename(ctx);
        self.output_dir.join(filename)
    }

    /// Renders a D2 diagram and returns the appropriate markdown events
    ///
    /// # Arguments
    /// * `ctx` - The render context for the diagram
    /// * `content` - The D2 diagram content
    pub fn render(
        &self,
        ctx: &RenderContext,
        content: &str,
    ) -> anyhow::Result<Vec<Event<'static>>> {
        fs::create_dir_all(Path::new(&self.source_dir).join(self.output_dir())).unwrap();

        let filepath = self.filepath(ctx);
        let args = [
            OsStr::new("--layout"),
            self.layout.as_ref(),
            OsStr::new("-"),
            filepath.as_os_str(),
        ];

        self.run_process(ctx, content, args)?;

        let depth = ctx.path.ancestors().count() - 2;
        let rel_path: PathBuf = std::iter::repeat(Path::new(".."))
            .take(depth)
            .collect::<PathBuf>()
            .join(self.relative_file_path(ctx));

        Ok(vec![
            Event::Start(Tag::Paragraph),
            Event::Start(Tag::Image {
                link_type: LinkType::Inline,
                dest_url: rel_path.to_string_lossy().to_string().into(),
                title: CowStr::Borrowed(""),
                id: CowStr::Borrowed(""),
            }),
            Event::End(TagEnd::Image),
            Event::End(TagEnd::Paragraph),
        ])
    }

    /// Runs the D2 process to generate a diagram
    ///
    /// # Arguments
    /// * `ctx` - The render context for the diagram
    /// * `content` - The D2 diagram content
    /// * `args` - Additional arguments for the D2 process
    fn run_process<I, S>(
        &self,
        ctx: &RenderContext,
        content: &str,
        args: I,
    ) -> anyhow::Result<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let child = Command::new(&self.path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(args)
            .spawn()?;

        child
            .stdin
            .as_ref()
            .unwrap()
            .write_all(content.as_bytes())?;

        let output = child.wait_with_output()?;
        if output.status.success() {
            let diagram = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(diagram)
        } else {
            let src =
                format!("\n{}", String::from_utf8_lossy(&output.stderr)).replace('\n', "\n  ");
            let msg = format!(
                "failed to compile D2 diagram ({}, #{}):{src}",
                ctx.chapter, ctx.diagram_index
            );
            bail!(msg)
        }
    }
}
