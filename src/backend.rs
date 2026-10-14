use std::{
    ffi::OsStr,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{bail, ensure, Context};
use mdbook_preprocessor::{book::SectionNumber, PreprocessorContext};
use pulldown_cmark::{CowStr, Event, LinkType, Tag, TagEnd};

use crate::{
    attributes::Attributes,
    config::{Config, Fonts},
};

/// Represents the backend for processing D2 diagrams
pub struct Backend {
    /// Absolute path to the D2 binary
    path: PathBuf,
    /// Relative path to the output directory for generated diagrams
    output_dir: PathBuf,
    /// Absolute path to the source directory of the book
    source_dir: PathBuf,
    /// Layout engine to use for D2 diagrams
    layout: Option<String>,
    inline: bool,
    /// Whether a diagram that fails to render should fail the whole build
    fail_on_error: bool,
    fonts: Option<Fonts>,
    theme_id: Option<String>,
    dark_theme_id: Option<String>,
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
            inline: config.inline,
            fail_on_error: config.fail_on_error,
            source_dir,
            fonts: config.fonts,
            theme_id: config.theme_id,
            dark_theme_id: config.dark_theme_id,
        }
    }

    /// Creates a Backend instance from a [`PreprocessorContext`]
    ///
    /// # Arguments
    /// * `ctx` - The preprocessor context
    pub fn from_context(ctx: &PreprocessorContext) -> Self {
        let config: Config = ctx
            .config
            .get("preprocessor.d2")
            .expect("Unable to deserialize d2 preprocessor config")
            .expect("d2 preprocessor config not found");
        let source_dir = ctx.root.join(&ctx.config.book.src);

        Self::new(config, source_dir)
    }

    /// Whether a diagram that fails to render should fail the whole build
    pub const fn fail_on_error(&self) -> bool {
        self.fail_on_error
    }

    /// Constructs the absolute file path for a diagram
    ///
    /// # Arguments
    /// * `ctx` - The render context for the diagram
    fn filepath(&self, ctx: &RenderContext) -> PathBuf {
        self.source_dir.join(self.relative_file_path(ctx))
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
    /// * `attributes` - The options set on the diagram's code block
    pub fn render(
        &self,
        ctx: &RenderContext,
        content: &str,
        attributes: &Attributes,
    ) -> anyhow::Result<Vec<Event<'static>>> {
        let diagram = self.compile(ctx, content, attributes)?;

        if self.inline {
            Ok(vec![Event::Html(
                format!("\n<pre>{diagram}</pre>\n").into(),
            )])
        } else {
            self.render_embedded(ctx, &diagram)
        }
    }

    fn render_embedded(
        &self,
        ctx: &RenderContext,
        diagram: &str,
    ) -> anyhow::Result<Vec<Event<'static>>> {
        let filepath = self.filepath(ctx);
        fs::create_dir_all(filepath.parent().expect("diagram path has a parent"))?;
        fs::write(&filepath, diagram)?;

        let depth = ctx.path.ancestors().count() - 2;
        let rel_path: PathBuf = std::iter::repeat_n(Path::new(".."), depth)
            .collect::<PathBuf>()
            .join(self.relative_file_path(ctx));

        Ok(vec![
            Event::Start(Tag::Paragraph),
            Event::Start(Tag::Image {
                link_type: LinkType::Inline,
                dest_url: rel_path
                    .to_string_lossy()
                    .to_string()
                    .replace('\\', "/")
                    .into(),
                title: CowStr::Borrowed(""),
                id: CowStr::Borrowed(""),
            }),
            Event::End(TagEnd::Image),
            Event::End(TagEnd::Paragraph),
        ])
    }

    /// Compiles a D2 diagram to a single SVG
    ///
    /// D2 is run against a temporary output file, rather than stdout, so that
    /// diagrams with multiple boards, or no content, can be reported clearly.
    fn compile(
        &self,
        ctx: &RenderContext,
        content: &str,
        attributes: &Attributes,
    ) -> anyhow::Result<String> {
        let dir = tempfile::tempdir()?;
        let output = dir.path().join("diagram.svg");

        self.run_process(ctx, content, self.args(attributes, &output))?;

        if !output.exists() {
            // a diagram with multiple boards is written as a directory of SVGs
            ensure!(
                !output.with_extension("").is_dir(),
                "D2 diagram ({}, #{}) has multiple boards (scenarios, steps or layers); select \
                 one to render with 'target=<board>'",
                ctx.chapter,
                ctx.diagram_index
            );

            // an empty diagram is compiled successfully, but writes nothing
            bail!(
                "D2 diagram ({}, #{}) is empty",
                ctx.chapter,
                ctx.diagram_index
            )
        }

        Ok(fs::read_to_string(output)?)
    }

    fn args<'a>(&'a self, attributes: &'a Attributes, output: &'a Path) -> Vec<&'a OsStr> {
        let mut args = vec![];

        if let Some(fonts) = &self.fonts {
            args.extend([
                OsStr::new("--font-regular"),
                fonts.regular.as_os_str(),
                OsStr::new("--font-italic"),
                fonts.italic.as_os_str(),
                OsStr::new("--font-bold"),
                fonts.bold.as_os_str(),
            ]);
        }
        if let Some(layout) = &self.layout {
            args.extend([OsStr::new("--layout"), layout.as_ref()]);
        }
        if let Some(theme_id) = &self.theme_id {
            args.extend([OsStr::new("--theme"), theme_id.as_ref()]);
        }
        if let Some(dark_theme_id) = &self.dark_theme_id {
            args.extend([OsStr::new("--dark-theme"), dark_theme_id.as_ref()]);
        }
        if let Some(target) = &attributes.target {
            args.extend([OsStr::new("--target"), target.as_ref()]);
        }
        args.extend([OsStr::new("-"), output.as_os_str()]);
        args
    }

    /// Runs the D2 process to generate a diagram
    ///
    /// # Arguments
    /// * `ctx` - The render context for the diagram
    /// * `content` - The D2 diagram content
    /// * `args` - Additional arguments for the D2 process
    fn run_process<I, S>(&self, ctx: &RenderContext, content: &str, args: I) -> anyhow::Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let child = Command::new(&self.path)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .args(args)
            .spawn()
            .with_context(|| {
                format!(
                    "failed to run D2 binary '{}' ({}, #{})",
                    self.path.display(),
                    ctx.chapter,
                    ctx.diagram_index
                )
            })?;

        child
            .stdin
            .as_ref()
            .unwrap()
            .write_all(content.as_bytes())
            .with_context(|| {
                format!(
                    "failed to write D2 diagram to stdin ({}, #{})",
                    ctx.chapter, ctx.diagram_index
                )
            })?;

        let output = child.wait_with_output().with_context(|| {
            format!(
                "failed to wait for D2 process ({}, #{})",
                ctx.chapter, ctx.diagram_index
            )
        })?;
        if output.status.success() {
            Ok(())
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
