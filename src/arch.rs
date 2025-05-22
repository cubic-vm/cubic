use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Eq, Hash, PartialEq, Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Arch {
    #[default]
    AMD64,
    ARM64,
}

impl Arch {
    pub fn from_str(arch: &str) -> Result<Arch, Error> {
        match arch {
            "amd64" => Ok(Arch::AMD64),
            "arm64" => Ok(Arch::ARM64),
            _ => Result::Err(Error::UnknownArch(arch.to_string())),
        }
    }
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Arch::AMD64 => write!(f, "amd64"),
            Arch::ARM64 => write!(f, "arm64"),
        }
    }
}
