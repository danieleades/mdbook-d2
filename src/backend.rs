use std::{
    ffi::OsStr,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use mdbook::{book::SectionNumber, preprocess::PreprocessorContext};
use pulldown_cmark::{CowStr, Event, LinkType, Tag};
use serde::Deserialize;

use crate::config::Config;

#[derive(Deserialize)]
#[serde(from = "Config")]
pub struct Backend {
    path: PathBuf,
    output_dir: PathBuf,
    layout: String,
}

impl From<Config> for Backend {
    fn from(config: Config) -> Self {
        Self {
            path: config.path,
            output_dir: config.output_dir,
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

    pub fn render(&self, ctx: RenderContext, content: &str) -> Vec<Event<'static>> {
        let filename = filename(&ctx);
        let filepath = Path::new("src").join(self.output_dir()).join(&filename);
        fs::create_dir_all(Path::new("src").join(self.output_dir())).unwrap();

        let mut child = Command::new(&self.path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args([
                OsStr::new("--layout"),
                self.layout.as_ref(),
                OsStr::new("-"),
                filepath.as_os_str(),
            ])
            .spawn()
            .expect("failed");

        child
            .stdin
            .take()
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        let output = child.wait_with_output().unwrap();
        if !output.status.success() {
            let src =
                format!("\n{}", String::from_utf8_lossy(&output.stderr)).replace('\n', "\n  ");
            let msg = format!(
                "failed to compile D2 diagram ({}, #{}):{src}",
                ctx.chapter, ctx.diagram_index
            );
            eprintln!("{msg}");
        }

        let depth = ctx.path.ancestors().count() - 1;
        let rel_path = PathBuf::from("../".repeat(depth))
            .join(self.output_dir())
            .join(filename);

        vec![
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
        ]
    }
}
