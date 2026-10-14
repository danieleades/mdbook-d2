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

    /// Animates all boards into a single SVG, showing each for this many
    /// milliseconds.
    pub animate_interval: Option<String>,
}

impl Attributes {
    /// Parses the attributes from a code block info string.
    ///
    /// Returns `None` if the code block is not a d2 diagram.
    pub fn from_info(info: &str) -> Option<anyhow::Result<Self>> {
        let (language, attributes) = info.split_once(char::is_whitespace).unwrap_or((info, ""));
        (language == "d2").then(|| Self::parse(attributes))
    }

    /// Parses whitespace-separated `key=value` attributes, where values may be
    /// quoted to include whitespace.
    fn parse(attributes: &str) -> anyhow::Result<Self> {
        let mut parsed = Self::default();

        for attribute in shlex::split(attributes).context("unbalanced quotes")? {
            let (key, value) = attribute
                .split_once('=')
                .with_context(|| format!("expected 'key=value', found '{attribute}'"))?;

            match key {
                "target" => parsed.target = Some(value.to_owned()),
                "animate-interval" => parsed.animate_interval = Some(value.to_owned()),
                _ => bail!("unknown attribute '{key}'"),
            }
        }

        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::Attributes;

    fn target(target: &str) -> Attributes {
        Attributes {
            target: Some(target.to_owned()),
            ..Attributes::default()
        }
    }

    fn animate_interval(interval: &str) -> Attributes {
        Attributes {
            animate_interval: Some(interval.to_owned()),
            ..Attributes::default()
        }
    }

    #[test_case("d2", &Attributes::default(); "bare")]
    #[test_case("d2 target=with_z", &target("with_z"); "target")]
    #[test_case("d2   target=a.b  ", &target("a.b"); "extra whitespace")]
    #[test_case(r#"d2 target="""#, &target(""); "quoted root")]
    #[test_case("d2 target=", &target(""); "empty root")]
    #[test_case(r#"d2 target="disaster recovery""#, &target("disaster recovery"); "quoted whitespace")]
    #[test_case("d2 animate-interval=1500", &animate_interval("1500"); "animate interval")]
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
    #[test_case(r#"d2 target="a b"#, "unbalanced quotes"; "unbalanced quotes")]
    fn rejects(info: &str, message: &str) {
        let err = Attributes::from_info(info).unwrap().unwrap_err();
        assert_eq!(err.to_string(), message);
    }
}
