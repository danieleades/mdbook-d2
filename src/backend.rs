use std::{
    ffi::OsStr,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::bail;
use mdbook_preprocessor::{book::SectionNumber, PreprocessorContext};
use pulldown_cmark::{CowStr, Event, LinkType, Tag, TagEnd};

use crate::config::{Config, Fonts};

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

/// For multiboard D2 output (e.g. diagrams with scenarios), D2 creates a
/// directory named after the requested output path (without the `.svg`
/// extension) containing `index.svg` and one `.svg` per board.
///
/// Returns the path to the actual SVG file to read or reference.
fn resolve_output_path(requested: &Path) -> PathBuf {
    let index = requested.with_extension("").join("index.svg");
    if index.exists() {
        index
    } else {
        requested.to_path_buf()
    }
}

fn cleanup_output_path(output_path: &Path) -> std::io::Result<()> {
    if output_path.exists() {
        fs::remove_file(output_path)?;
    }

    let output_dir = output_path.with_extension("");
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)?;
    }

    Ok(())
}

fn copy_dir(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());

        if entry.file_type()?.is_dir() {
            copy_dir(&source_path, &destination_path)?;
        } else {
            fs::copy(source_path, destination_path)?;
        }
    }

    Ok(())
}

fn svg_names(dir: &Path) -> std::io::Result<Vec<String>> {
    let mut svg_names = fs::read_dir(dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            let is_svg = path.extension().is_some_and(|extension| extension == "svg");

            if entry.file_type().ok()?.is_file() && is_svg {
                Some(entry.file_name().to_string_lossy().into_owned())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    svg_names.sort();
    Ok(svg_names)
}

fn rewrite_svg_links(diagram: &str, asset_dir: &Path, svg_names: &[String]) -> String {
    let asset_dir = asset_dir.to_string_lossy().replace('\\', "/");
    let mut diagram = diagram.to_owned();

    for svg_name in svg_names {
        let target = format!("{asset_dir}/{svg_name}");

        for attribute in ["href", "xlink:href"] {
            diagram = diagram.replace(
                &format!(r#"{attribute}="{svg_name}""#),
                &format!(r#"{attribute}="{target}""#),
            );
            diagram = diagram.replace(
                &format!(r"{attribute}='{svg_name}'"),
                &format!(r"{attribute}='{target}'"),
            );
        }
    }

    diagram
}

fn strip_xml_declaration(diagram: &str) -> &str {
    let diagram = diagram.trim_start();

    if !diagram.starts_with("<?xml") {
        return diagram;
    }

    diagram
        .split_once("?>")
        .map_or(diagram, |(_, diagram)| diagram.trim_start())
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
    pub fn render(
        &self,
        ctx: &RenderContext,
        content: &str,
    ) -> anyhow::Result<Vec<Event<'static>>> {
        if self.inline {
            self.render_inline(ctx, content)
        } else {
            self.render_embedded(ctx, content)
        }
    }

    fn compile(
        &self,
        ctx: &RenderContext,
        content: &str,
        output_path: &Path,
    ) -> anyhow::Result<()> {
        fs::create_dir_all(output_path.parent().expect("output path has no parent"))?;
        cleanup_output_path(output_path)?;
        let mut args = self.basic_args();
        args.push(output_path.as_os_str());
        self.run_process(ctx, content, args)?;
        Ok(())
    }

    fn relative_path(&self, ctx: &RenderContext, target: &Path) -> PathBuf {
        let rel_from_source = target
            .strip_prefix(&self.source_dir)
            .expect("output path is within source dir");

        let depth = ctx.path.ancestors().count() - 2;
        std::iter::repeat_n(Path::new(".."), depth)
            .collect::<PathBuf>()
            .join(rel_from_source)
    }

    fn render_inline_svg(diagram: &str) -> Vec<Event<'static>> {
        let diagram = strip_xml_declaration(diagram);
        vec![Event::Html(
            format!("\n<div class=\"mdbook-d2\">\n{diagram}\n</div>\n").into(),
        )]
    }

    fn render_image(rel_path: &Path) -> Vec<Event<'static>> {
        vec![
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
        ]
    }

    fn render_multiboard(
        &self,
        ctx: &RenderContext,
        output_path: &Path,
    ) -> anyhow::Result<Vec<Event<'static>>> {
        let actual = resolve_output_path(output_path);
        let output_dir = actual.parent().expect("multiboard output has no parent");
        let diagram = fs::read_to_string(&actual)?;
        let svg_names = svg_names(output_dir)?;
        let rel_output_dir = self.relative_path(ctx, output_dir);

        let diagram = rewrite_svg_links(&diagram, &rel_output_dir, &svg_names);
        Ok(Self::render_inline_svg(&diagram))
    }

    fn publish_multiboard_output(source: &Path, destination: &Path) -> anyhow::Result<()> {
        fs::create_dir_all(destination.parent().expect("output path has no parent"))?;
        cleanup_output_path(destination)?;
        copy_dir(&source.with_extension(""), &destination.with_extension(""))?;
        Ok(())
    }

    fn render_inline(
        &self,
        ctx: &RenderContext,
        content: &str,
    ) -> anyhow::Result<Vec<Event<'static>>> {
        let tmp_dir = tempfile::tempdir()?;
        let output_path = tmp_dir.path().join("output.svg");
        self.compile(ctx, content, &output_path)?;
        let actual = resolve_output_path(&output_path);

        if actual == output_path {
            let diagram = fs::read_to_string(actual)?;
            return Ok(Self::render_inline_svg(&diagram));
        }

        let published_path = self.filepath(ctx);
        Self::publish_multiboard_output(&output_path, &published_path)?;
        self.render_multiboard(ctx, &published_path)
    }

    fn render_embedded(
        &self,
        ctx: &RenderContext,
        content: &str,
    ) -> anyhow::Result<Vec<Event<'static>>> {
        let filepath = self.filepath(ctx);
        self.compile(ctx, content, &filepath)?;

        let actual = resolve_output_path(&filepath);

        if actual == filepath {
            return Ok(Self::render_image(&self.relative_path(ctx, &actual)));
        }

        self.render_multiboard(ctx, &filepath)
    }

    fn basic_args(&self) -> Vec<&OsStr> {
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
        args.push(OsStr::new("-"));
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{rewrite_svg_links, strip_xml_declaration};

    #[test]
    fn rewrite_svg_links_only_updates_known_board_assets() {
        let diagram = concat!(
            r#"<a href="with_z.svg"></a>"#,
            r#"<a xlink:href='index.svg'></a>"#,
            r##"<a href="#local"></a>"##
        );

        let rewritten = rewrite_svg_links(
            diagram,
            Path::new("d2/1.1"),
            &["index.svg".into(), "with_z.svg".into()],
        );

        assert!(rewritten.contains(r#"href="d2/1.1/with_z.svg""#));
        assert!(rewritten.contains(r"xlink:href='d2/1.1/index.svg'"));
        assert!(rewritten.contains(r##"href="#local""##));
    }

    #[test]
    fn strip_xml_declaration_removes_svg_preamble() {
        let diagram = "<?xml version=\"1.0\"?><svg></svg>";

        assert_eq!(strip_xml_declaration(diagram), "<svg></svg>");
    }
}
