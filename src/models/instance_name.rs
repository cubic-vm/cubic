use regex::Regex;
use std::fmt;
use std::str::FromStr;
use std::sync::LazyLock;

static INSTANCE_NAME_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new("^[\\w_-]+$").unwrap());

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InstanceName {
    name: String,
}

impl InstanceName {
    pub fn as_str(&self) -> &str {
        self.name.as_str()
    }
}

impl FromStr for InstanceName {
    type Err = String;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        if INSTANCE_NAME_REGEX.is_match(name) {
            Ok(Self {
                name: name.to_string(),
            })
        } else {
            Err(
                "Instance name must only contain letters, numbers, underlines and dashes"
                    .to_string(),
            )
        }
    }
}

impl fmt::Display for InstanceName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accept_a_valid_name() {
        for input in ["foobar", "12345", "_", "-", "10foo-bar_5", "café"] {
            InstanceName::from_str(input).unwrap();
        }
    }

    #[test]
    fn test_reject_an_invalid_name() {
        assert!(InstanceName::from_str("foo/bar").is_err());
        assert!(InstanceName::from_str("/abs/path").is_err());
        assert!(InstanceName::from_str("").is_err());
    }

    #[test]
    fn test_reject_path_traversal() {
        assert!(InstanceName::from_str("../../etc").is_err());
    }
}
