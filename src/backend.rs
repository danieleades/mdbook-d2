use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use anyhow::bail;
use mdbook::book::SectionNumber;
use mdbook::preprocess::PreprocessorContext;
use pulldown_cmark::{CowStr, Event, LinkType, Tag};
use serde::Deserialize;

use crate::config::Config;

#[derive(Deserialize)]
#[serde(from = "Config")]
pub struct Backend {
    path: PathBuf,
    output_dir: PathBuf,
    inline: bool,
    layout: String,
}

impl From<Config> for Backend {
    fn from(config: Config) -> Self {
        Self {
            path: config.path,
            output_dir: config.output_dir,
            inline: config.inline,
            layout: config.layout,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RenderContext<'a> {
    path: &'a Path,
    chapter: &'a str,
    section: Option<&'a SectionNumber>,
    diagram_index: usize,
}

impl<'a> RenderContext<'a> {
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

fn filename(ctx: &RenderContext) -> String {
    format!(
        "{}{}.svg",
        ctx.section.cloned().unwrap_or_default(),
        ctx.diagram_index
    )
}

impl Backend {
    pub fn from_context(ctx: &PreprocessorContext) -> Self {
        let value: toml::Value = ctx.config.get_preprocessor("d2").unwrap().clone().into();
        value.try_into().unwrap()
    }

    fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    fn filepath(&self, ctx: &RenderContext) -> PathBuf {
        Path::new("src").join(self.relative_file_path(ctx))
    }

    fn relative_file_path(&self, ctx: &RenderContext) -> PathBuf {
        let filename = filename(ctx);
        self.output_dir().join(filename)
    }

    fn generate_d2_file(&self, ctx: &RenderContext, content: &str) -> anyhow::Result<()> {
        let child = Command::new(&self.path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args([
                OsStr::new("--layout"),
                self.layout.as_ref(),
                OsStr::new("-"),
                self.filepath(ctx).as_os_str(),
            ])
            .spawn()?;

        run_process(child, ctx, content)?;
        Ok(())
    }

    fn generate_d2_string(&self, ctx: &RenderContext, content: &str) -> anyhow::Result<String> {
        let child = Command::new(&self.path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args([
                OsStr::new("--layout"),
                self.layout.as_ref(),
                OsStr::new("-"),
            ])
            .spawn()?;

        run_process(child, ctx, content)
    }

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

    fn render_embedded(
        &self,
        ctx: &RenderContext,
        content: &str,
    ) -> anyhow::Result<Vec<Event<'static>>> {
        fs::create_dir_all(Path::new("src").join(self.output_dir())).unwrap();

        self.generate_d2_file(ctx, content)?;

        let depth = ctx.path.ancestors().count() - 2;
        let rel_path: PathBuf = std::iter::repeat(Path::new(".."))
            .take(depth)
            .collect::<PathBuf>()
            .join(self.relative_file_path(ctx));

        Ok(vec![
            Event::Start(Tag::Image(
                LinkType::Inline,
                rel_path.to_string_lossy().to_string().into(),
                CowStr::Borrowed(""),
            )),
            Event::End(Tag::Image(
                LinkType::Inline,
                rel_path.to_string_lossy().to_string().into(),
                CowStr::Borrowed(""),
            )),
        ])
    }

    fn render_inline(
        &self,
        ctx: &RenderContext,
        content: &str,
    ) -> anyhow::Result<Vec<Event<'static>>> {
        let diagram = self.generate_d2_string(ctx, content)?;

        let s = format!("<pre>{diagram}</pre>");

        Ok(vec![Event::Html(s.into())])
    }
}

fn run_process(child: Child, ctx: &RenderContext, content: &str) -> anyhow::Result<String> {
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
        let src = format!("\n{}", String::from_utf8_lossy(&output.stderr)).replace('\n', "\n  ");
        let msg = format!(
            "failed to compile D2 diagram ({}, #{}):{src}",
            ctx.chapter, ctx.diagram_index
        );
        bail!(msg)
    }
}
