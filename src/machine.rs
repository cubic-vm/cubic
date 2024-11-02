pub mod machine_dao;

pub use machine_dao::*;
use serde::{Deserialize, Serialize};

fn default_user() -> String {
    USER.to_string()
}

#[derive(PartialEq, Default, Debug, Clone, Serialize, Deserialize)]
pub struct MountPoint {
    pub host: String,
    pub guest: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Machine {
    #[serde(skip)]
    pub name: String,
    #[serde(default = "default_user")]
    pub user: String,
    pub cpus: u16,
    pub mem: u64,
    pub disk_capacity: u64,
    pub ssh_port: u16,
    #[serde(default)]
    pub display: bool,
    #[serde(default)]
    pub gpu: bool,
    #[serde(default)]
    pub mounts: Vec<MountPoint>,
    #[serde(default)]
    pub hostfwd: Vec<String>,
}
