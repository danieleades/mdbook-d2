use std::path::PathBuf;

use serde::Deserialize;

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

#[derive(Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(default = "default::bin_path")]
    pub path: PathBuf,

    #[serde(default = "default::output_dir")]
    pub output_dir: PathBuf,

    #[serde(default = "default::layout")]
    pub layout: String,

    #[serde(default)]
    pub file_extension: FileExtension,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: PathBuf::from("d2"),
            layout: String::from("dagre"),
            output_dir: PathBuf::from("d2"),
            file_extension: FileExtension::default(),
        }
    }
}

#[derive(Default, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum FileExtension {
    #[default]
    Svg,
    Png,
}

impl AsRef<str> for FileExtension {
    fn as_ref(&self) -> &str {
        match self {
            Self::Svg => "svg",
            Self::Png => "png",
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use test_case::test_case;

    use super::{Config, FileExtension};

    #[test_case(""; "empty")]
    #[test_case(
        r#"
path = "d2"
layout = "dagre"
output-dir = "d2"
file-extension = "svg"
"#
        ; "defaults"
    )]
    fn compatible(input: &str) {
        let _config: Config = toml::from_str(input).expect("config is not compatible");
    }

    #[test_case("" => Config::default(); "default")]
    #[test_case(
        r#"
path = "/custom/bin/d2"
layout = "elk"
output-dir = "d2-img"
file-extension = "png"
"#
    => Config {
        path: PathBuf::from("/custom/bin/d2"),
        layout: String::from("elk"),
        output_dir: PathBuf::from("d2-img"),
        file_extension: FileExtension::Png,

    }
        ; "custom"
    )]
    fn parse(input: &str) -> Config {
        toml::from_str(input).unwrap()
    }
}
