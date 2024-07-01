pub mod machine_dao;

pub use machine_dao::*;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Default, Debug, Clone, Serialize, Deserialize)]
pub struct MountPoint {
    pub host: String,
    pub guest: String,
}

pub struct Machine {
    pub name: String,
    pub cpus: u16,
    pub mem: u64,
    pub disk_capacity: u64,
    pub ssh_port: u16,
    pub sandbox: bool,
    pub mounts: Vec<MountPoint>,
}
