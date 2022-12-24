use std::{
    ffi::OsStr,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use pulldown_cmark::{CowStr, Event, LinkType, Tag};
use serde::Deserialize;

fn d2_default_binary_path() -> PathBuf {
    PathBuf::from("d2")
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("d2")
}

#[derive(Deserialize)]
pub struct Backend {
    #[serde(default = "d2_default_binary_path")]
    path: PathBuf,

    #[serde(default = "default_output_dir")]
    output_dir: PathBuf,
}

impl Backend {
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
            .args([OsStr::new("-"), filepath.as_os_str()])
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
