use serde::Deserialize;
use std::path::PathBuf;

mod default {
    use std::path::PathBuf;

    pub fn bin_path() -> PathBuf {
        PathBuf::from("d2")
    }

    pub fn output_dir() -> PathBuf {
        PathBuf::from("d2")
    }

    pub fn layout() -> String {
        String::from("dagre")
    }
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "default::bin_path")]
    pub path: PathBuf,

    #[serde(default = "default::output_dir")]
    pub output_dir: PathBuf,

    #[serde(default = "default::layout")]
    pub layout: String,
}
