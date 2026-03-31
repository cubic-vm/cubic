use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Eq, Hash, PartialEq, Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Arch {
    #[default]
    AMD64,
    ARM64,
}

impl Arch {
    pub fn from_str(arch: &str) -> Result<Arch> {
        match arch {
            "amd64" => Ok(Arch::AMD64),
            "arm64" => Ok(Arch::ARM64),
            _ => Result::Err(Error::UnknownArch(arch.to_string())),
        }
    }

    pub fn as_vendor_str(&self) -> &str {
        match &self {
            Arch::AMD64 => "amd64",
            Arch::ARM64 => "arm64",
        }
    }

    pub fn as_canonical_str(&self) -> &str {
        match &self {
            Arch::AMD64 => "x86_64",
            Arch::ARM64 => "aarch64",
        }
    }
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_vendor_str())
    }
}
