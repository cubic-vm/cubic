use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Eq, Hash, PartialEq, Default, Debug, Clone, Serialize, Deserialize)]
pub enum Arch {
    #[default]
    AMD64,
    ARM64,
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Arch::AMD64 => write!(f, "amd64"),
            Arch::ARM64 => write!(f, "arm64"),
        }
    }
}
