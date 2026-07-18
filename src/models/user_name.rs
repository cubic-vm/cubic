use crate::error::Error;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;
use std::sync::LazyLock;

pub const DEFAULT_USERNAME: &str = "cubic";

static USER_NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^[a-z_][a-z0-9_-]*$").unwrap());

#[derive(Clone, Debug, PartialEq)]
pub struct UserName {
    name: String,
}

impl UserName {
    pub fn as_str(&self) -> &str {
        self.name.as_str()
    }
}

impl Default for UserName {
    fn default() -> Self {
        Self {
            name: DEFAULT_USERNAME.to_string(),
        }
    }
}

impl FromStr for UserName {
    type Err = Error;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        if USER_NAME_REGEX.is_match(name) {
            Ok(Self {
                name: name.to_string(),
            })
        } else {
            Err(Error::InvalidUsername(name.to_string()))
        }
    }
}

impl fmt::Display for UserName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.name)
    }
}

impl Serialize for UserName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.name)
    }
}

impl<'de> Deserialize<'de> for UserName {
    // Falls back to the default username instead of failing so that a persisted
    // instance config with a username predating this validation (or edited by
    // hand) does not take down the rest of the instance's settings with it.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = String::deserialize(deserializer)?;
        Ok(UserName::from_str(&name).unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_letters() {
        UserName::from_str("tux").unwrap();
    }

    #[test]
    fn test_leading_underscore() {
        UserName::from_str("_tux").unwrap();
    }

    #[test]
    fn test_numbers_underscore_dash_after_first_char() {
        UserName::from_str("tux-01_x").unwrap();
    }

    #[test]
    fn test_reject_leading_digit() {
        assert!(UserName::from_str("1tux").is_err());
    }

    #[test]
    fn test_reject_leading_dash() {
        assert!(UserName::from_str("-tux").is_err());
    }

    #[test]
    fn test_reject_empty_name() {
        assert!(UserName::from_str("").is_err());
    }

    #[test]
    fn test_reject_space() {
        assert!(UserName::from_str("bad name").is_err());
    }

    #[test]
    fn test_reject_newline() {
        assert!(UserName::from_str("tux\nroot").is_err());
    }

    #[test]
    fn test_reject_yaml_injection() {
        assert!(UserName::from_str("tux\n  - name: root").is_err());
    }

    #[test]
    fn test_reject_uppercase() {
        assert!(UserName::from_str("Tux").is_err());
    }

    #[test]
    fn test_serialize_round_trip() {
        let user = UserName::from_str("tux").unwrap();
        let json = serde_json::to_string(&user).unwrap();
        assert_eq!(json, "\"tux\"");
        let back: UserName = serde_json::from_str(&json).unwrap();
        assert_eq!(back, user);
    }

    #[test]
    fn test_deserialize_falls_back_to_default_for_invalid_value() {
        let result: UserName = serde_json::from_str("\"bad name\"").unwrap();
        assert_eq!(result, UserName::default());
    }
}
