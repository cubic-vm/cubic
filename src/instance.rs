mod instance_dao;
mod instance_deserializer;
mod instance_name;
mod instance_serializer;
mod instance_store;
mod instance_store_mock;
mod port_forward;
mod target;
mod target_instance_path;
mod target_path;
mod toml_instance_deserializer;
mod yaml_instance_deserializer;

use crate::arch::Arch;
pub use instance_dao::*;
pub use instance_deserializer::*;
pub use instance_name::*;
pub use instance_serializer::*;
pub use instance_store::*;
#[cfg(test)]
pub use instance_store_mock::tests::InstanceStoreMock;
pub use port_forward::*;
use serde::{Deserialize, Serialize};
pub use target::*;
pub use target_instance_path::*;
pub use target_path::*;
pub use toml_instance_deserializer::*;
pub use yaml_instance_deserializer::*;

use crate::env::DEFAULT_USERNAME;
use crate::model::DataSize;

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
