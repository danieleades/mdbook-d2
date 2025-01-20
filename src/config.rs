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

    pub const fn inline() -> bool {
        true
    }
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct Fonts {
    pub regular: PathBuf,
    pub italic: PathBuf,
    pub bold: PathBuf,
}
#[derive(Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// The path to the d2 binary
    #[serde(default = "default::bin_path")]
    pub path: PathBuf,

    #[serde(default = "default::output_dir")]
    pub output_dir: PathBuf,

    pub layout: Option<String>,

    /// Whether or not to use inline SVG when building an HTML target
    ///
    /// Default is 'true'
    #[serde(default = "default::inline")]
    pub inline: bool,

    /// Custom font path
    ///
    /// Only ttf fonts are valid
    pub fonts: Option<Fonts>,

    pub theme_id: Option<String>,
    pub dark_theme_id: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: default::bin_path(),
            layout: None,
            output_dir: default::output_dir(),
            inline: default::inline(),
            fonts: None,
            theme_id: None,
            dark_theme_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use test_case::test_case;

    use super::Config;

    #[test_case(""; "empty")]
    #[test_case(
        r#"
path = "d2"
layout = "dagre"
output-dir = "d2"
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
"#
    => Config {
        path: PathBuf::from("/custom/bin/d2"),
        layout: Some(String::from("elk")),
        inline: true,
        output_dir: PathBuf::from("d2-img"),
        fonts: None,
        theme_id: None,
        dark_theme_id:None,
    }
        ; "custom"
    )]
    fn parse(input: &str) -> Config {
        toml::from_str(input).unwrap()
    }
}
