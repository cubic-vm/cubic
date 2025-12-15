pub mod instance_dao;
pub mod instance_deserializer;
pub mod instance_name;
pub mod instance_store;
pub mod instance_store_mock;
pub mod port_forward;
pub mod target;
pub mod target_instance_path;
pub mod target_path;
pub mod yaml_instance_deserializer;

use crate::arch::Arch;
pub use crate::error::Error;
pub use instance_dao::*;
pub use instance_deserializer::*;
pub use instance_name::*;
pub use instance_store::*;
pub use port_forward::*;
use serde::{Deserialize, Serialize};
use std::io::Write;
pub use target::*;
pub use target_instance_path::*;
pub use target_path::*;
pub use yaml_instance_deserializer::*;

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
    pub mem: u64,
    #[serde(skip)]
    pub disk_used: Option<u64>,
    pub disk_capacity: u64,
    pub ssh_port: u16,
    #[serde(default)]
    pub hostfwd: Vec<PortForward>,
}

impl Instance {
    pub fn serialize(&self, writer: &mut dyn Write) -> Result<(), Error> {
        serde_yaml::to_writer(
            writer,
            &Config {
                machine: self.clone(),
            },
        )
        .map_err(Error::SerdeYaml)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    

    #[test]
    fn test_serialize_minimal_config() {
        let mut writer = Vec::new();

        Instance {
            name: "test".to_string(),
            arch: Arch::AMD64,
            user: "tux".to_string(),
            cpus: 1,
            mem: 1000,
            disk_capacity: 1000,
            ssh_port: 10000,
            hostfwd: Vec::new(),
            ..Instance::default()
        }
        .serialize(&mut writer)
        .expect("Cannot parser config");
        let config = String::from_utf8(writer).unwrap();

        assert_eq!(
            config,
            r#"machine:
  arch: AMD64
  user: tux
  cpus: 1
  mem: 1000
  disk_capacity: 1000
  ssh_port: 10000
  hostfwd: []
"#
        );
    }
}
