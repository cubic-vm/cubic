pub mod machine_dao;

pub use machine_dao::*;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Default, Debug, Clone, Serialize, Deserialize)]
pub struct MountPoint {
    pub host: String,
    pub guest: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Machine {
    #[serde(skip)]
    pub name: String,
    pub cpus: u16,
    pub mem: u64,
    pub disk_capacity: u64,
    pub ssh_port: u16,
    #[serde(default)]
    pub sandbox: bool,
    #[serde(default)]
    pub mounts: Vec<MountPoint>,
}
