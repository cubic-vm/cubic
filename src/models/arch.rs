use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Eq, Hash, PartialEq, Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Arch {
    #[default]
    AMD64,
    ARM64,
}

impl Arch {
    pub fn as_vendor_str(&self) -> &str {
        match self {
            Arch::AMD64 => "amd64",
            Arch::ARM64 => "arm64",
        }
    }

    pub fn as_canonical_str(&self) -> &str {
        match self {
            Arch::AMD64 => "x86_64",
            Arch::ARM64 => "aarch64",
        }
    }
}

impl FromStr for Arch {
    type Err = Error;

    fn from_str(arch: &str) -> Result<Arch> {
        match arch {
            "amd64" => Ok(Arch::AMD64),
            "arm64" => Ok(Arch::ARM64),
            _ => Err(Error::UnknownArch(arch.to_string())),
        }
    }
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_vendor_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_amd64() {
        assert_eq!(Arch::from_str("amd64").unwrap(), Arch::AMD64);
    }

    #[test]
    fn test_parse_arm64() {
        assert_eq!(Arch::from_str("arm64").unwrap(), Arch::ARM64);
    }

    #[test]
    fn test_reject_unknown_arch() {
        assert!(matches!(
            Arch::from_str("mips"),
            Err(Error::UnknownArch(ref arch)) if arch == "mips"
        ));
    }

    #[test]
    fn test_vendor_str() {
        assert_eq!(Arch::AMD64.as_vendor_str(), "amd64");
        assert_eq!(Arch::ARM64.as_vendor_str(), "arm64");
    }

    #[test]
    fn test_canonical_str() {
        assert_eq!(Arch::AMD64.as_canonical_str(), "x86_64");
        assert_eq!(Arch::ARM64.as_canonical_str(), "aarch64");
    }

    #[test]
    fn test_to_string() {
        assert_eq!(Arch::AMD64.to_string(), "amd64");
        assert_eq!(Arch::ARM64.to_string(), "arm64");
    }
}
