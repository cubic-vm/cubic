pub mod instance_dao;
pub mod instance_deserializer;
pub mod instance_name;
pub mod instance_serializer;
pub mod instance_store;
pub mod instance_store_mock;
pub mod port_forward;
pub mod target;
pub mod target_instance_path;
pub mod target_path;
pub mod toml_instance_deserializer;
pub mod yaml_instance_deserializer;

use crate::arch::Arch;
pub use instance_dao::*;
pub use instance_deserializer::*;
pub use instance_name::*;
pub use instance_serializer::*;
pub use instance_store::*;
pub use port_forward::*;
use serde::{Deserialize, Serialize};
pub use target::*;
pub use target_instance_path::*;
pub use target_path::*;
pub use toml_instance_deserializer::*;
pub use yaml_instance_deserializer::*;

use crate::model::DataSize;

fn default_user() -> String {
    USER.to_string()
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
}
