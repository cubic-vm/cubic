use crate::env::DEFAULT_USERNAME;
use crate::models::{Arch, DataSize, PortForward};
use serde::{Deserialize, Serialize};

fn default_user() -> String {
    DEFAULT_USERNAME.to_string()
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instance {
    #[serde(skip)]
    pub name: String,
    #[serde(default)]
    pub arch: Arch,
    #[serde(default = "default_user")]
    pub user: String,
    pub cpus: u16,
    pub mem: DataSize,
    #[serde(skip)]
    pub disk_used: Option<DataSize>,
    pub disk_capacity: DataSize,
    pub ssh_port: u16,
    #[serde(default)]
    pub hostfwd: Vec<PortForward>,
    #[serde(default)]
    pub execute: Option<String>,
    #[serde(default)]
    pub isolate: bool,
}
