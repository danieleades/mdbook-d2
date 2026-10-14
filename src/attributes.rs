use anyhow::{bail, Context};

/// Per-diagram options, set on the code block's info string.
///
/// ````md
/// ```d2 target=scenarios.failover
/// ...
/// ```
/// ````
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Attributes {
    /// The board to render, for diagrams with scenarios, steps or layers.
    ///
    /// An empty target selects the root board.
    pub target: Option<String>,
}

impl Attributes {
    /// Parses the attributes from a code block info string.
    ///
    /// Returns `None` if the code block is not a d2 diagram.
    pub fn from_info(info: &str) -> Option<anyhow::Result<Self>> {
        let mut words = info.split_whitespace();
        (words.next() == Some("d2")).then(|| Self::parse(words))
    }

    fn parse<'a>(attributes: impl IntoIterator<Item = &'a str>) -> anyhow::Result<Self> {
        let mut parsed = Self::default();

        for attribute in attributes {
            let (key, value) = attribute
                .split_once('=')
                .with_context(|| format!("expected 'key=value', found '{attribute}'"))?;

            match key {
                "target" => parsed.target = Some(unquote(value).to_owned()),
                _ => bail!("unknown attribute '{key}'"),
            }
        }

        Ok(parsed)
    }
}

/// Strips a single pair of surrounding double quotes, if present.
fn unquote(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::Attributes;

    fn target(target: &str) -> Attributes {
        Attributes {
            target: Some(target.to_owned()),
        }
    }

    #[test_case("d2", &Attributes::default(); "bare")]
    #[test_case("d2 target=with_z", &target("with_z"); "target")]
    #[test_case("d2   target=a.b  ", &target("a.b"); "extra whitespace")]
    #[test_case(r#"d2 target="""#, &target(""); "quoted root")]
    #[test_case("d2 target=", &target(""); "empty root")]
    fn parses(info: &str, expected: &Attributes) {
        assert_eq!(&Attributes::from_info(info).unwrap().unwrap(), expected);
    }

    #[test_case("rust"; "other language")]
    #[test_case("d2x"; "language prefix")]
    #[test_case(""; "no language")]
    fn ignores_other_code_blocks(info: &str) {
        assert!(Attributes::from_info(info).is_none());
    }

    #[test_case("d2 target", "expected 'key=value', found 'target'"; "missing value")]
    #[test_case("d2 colour=red", "unknown attribute 'colour'"; "unknown key")]
    fn rejects(info: &str, message: &str) {
        let err = Attributes::from_info(info).unwrap().unwrap_err();
        assert_eq!(err.to_string(), message);
    }
}
