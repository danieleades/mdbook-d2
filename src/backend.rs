use std::{
    ffi::OsStr,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use mdbook::preprocess::PreprocessorContext;
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

impl Backend {
    pub fn from_context(ctx: &PreprocessorContext) -> Self {
        let value: toml::Value = ctx.config.get_preprocessor("d2").unwrap().clone().into();
        value.try_into().unwrap()
    }

    fn output_dir(&self) -> PathBuf {
        Path::new("src").join(&self.output_dir)
    }

    pub fn render(
        &self,
        chapter: &str,
        diagram_index: usize,
        content: &str,
    ) -> Vec<Event<'static>> {
        let filename = format!("{chapter}-{diagram_index}.svg");
        let filepath = self.output_dir().join(&filename);
        fs::create_dir_all(self.output_dir()).unwrap();

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
            let msg = format!("failed to compile D2 diagram ({chapter}, #{diagram_index}):{src}");
            eprintln!("{msg}");
        }

        let rel_path = format!("d2/{filename}");

        vec![
            Event::Start(Tag::Image(
                LinkType::Inline,
                rel_path.clone().into(),
                CowStr::Borrowed(""),
            )),
            Event::End(Tag::Image(
                LinkType::Inline,
                rel_path.into(),
                CowStr::Borrowed(""),
            )),
        ]
    }
}
