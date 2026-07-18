use crate::models::{Arch, DataSize, PortForward, UserName};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instance {
    #[serde(skip)]
    pub name: String,
    #[serde(default)]
    pub arch: Arch,
    #[serde(default)]
    pub user: UserName,
    pub cpus: u16,
    pub mem: DataSize,
    #[serde(skip)]
    pub disk_used: Option<DataSize>,
    pub disk_capacity: DataSize,
    pub ssh_port: u16,
    #[serde(default)]
    pub monitor_port: Option<u16>,
    #[serde(default)]
    pub console_port: Option<u16>,
    #[serde(default)]
    pub hostfwd: Vec<PortForward>,
    #[serde(default)]
    pub execute: Option<String>,
    #[serde(default)]
    pub isolate: bool,
}
