pub mod instance_dao;
pub mod instance_state;
pub mod instance_store;
pub mod instance_store_mock;

use crate::arch::Arch;
pub use crate::error::Error;
pub use instance_dao::*;
pub use instance_state::*;
pub use instance_store::*;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

fn default_user() -> String {
    USER.to_string()
}

#[derive(PartialEq, Default, Debug, Clone, Serialize, Deserialize)]
pub struct MountPoint {
    pub host: String,
    pub guest: String,
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

impl Instance {
    pub fn deserialize(name: &str, reader: &mut dyn Read) -> Result<Instance, Error> {
        serde_yaml::from_reader(reader)
            .map(|config: Config| config.machine)
            .map(|mut instance: Instance| {
                instance.name = name.to_string();
                instance
            })
            .map_err(|_| Error::CannotParseFile(String::new()))
    }

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

    use std::io::BufReader;

    #[test]
    fn test_deserialize_empty_file() {
        let reader = &mut BufReader::new("".as_bytes());
        let instance = Instance::deserialize("test", reader);
        assert!(instance.is_err());
    }

    #[test]
    fn test_deserialize_minimal_config() {
        let reader = &mut BufReader::new(
            r#"
machine:
  cpus: 1
  mem: 1073741824
  disk_capacity: 2361393152
  ssh_port: 14357
"#
            .as_bytes(),
        );

        let instance = Instance::deserialize("test", reader).expect("Cannot parser config");
        assert_eq!(instance.name, "test");
        assert_eq!(instance.user, "cubic");
        assert_eq!(instance.cpus, 1);
        assert_eq!(instance.mem, 1073741824);
        assert_eq!(instance.disk_capacity, 2361393152);
        assert_eq!(instance.ssh_port, 14357);
        assert!(!instance.display);
        assert!(!instance.gpu);
        assert!(instance.mounts.is_empty());
        assert!(instance.hostfwd.is_empty());
    }

    #[test]
    fn test_deserialize_full_config() {
        let reader = &mut BufReader::new(
            r#"
machine:
  user: tux
  cpus: 1
  mem: 1073741824
  disk_capacity: 2361393152
  ssh_port: 14357
  display: false
  gpu: false
  mounts:
    - host: /home/tux/guest
      guest: /home/tux
  hostfwd: ["tcp:127.0.0.1:8000-:8000", "tcp:127.0.0.1:9000-:10000"]
"#
            .as_bytes(),
        );

        let instance = Instance::deserialize("test", reader).expect("Cannot parser config");
        assert_eq!(instance.name, "test");
        assert_eq!(instance.user, "tux");
        assert_eq!(instance.cpus, 1);
        assert_eq!(instance.mem, 1073741824);
        assert_eq!(instance.disk_capacity, 2361393152);
        assert_eq!(instance.ssh_port, 14357);
        assert!(!instance.display);
        assert!(!instance.gpu);
        assert_eq!(
            instance.mounts,
            [MountPoint {
                host: "/home/tux/guest".to_string(),
                guest: "/home/tux".to_string()
            }]
        );
        assert_eq!(
            instance.hostfwd,
            ["tcp:127.0.0.1:8000-:8000", "tcp:127.0.0.1:9000-:10000"]
        );
    }

    #[test]
    fn test_deserialize_desktop_config() {
        let reader = &mut BufReader::new(
            r#"
machine:
  user: tux
  cpus: 1
  mem: 1073741824
  disk_capacity: 2361393152
  ssh_port: 14357
  display: true
  gpu: true
  mounts:
  hostfwd:
"#
            .as_bytes(),
        );

        let instance = Instance::deserialize("test", reader).expect("Cannot parser config");
        assert_eq!(instance.name, "test");
        assert_eq!(instance.user, "tux");
        assert_eq!(instance.cpus, 1);
        assert_eq!(instance.mem, 1073741824);
        assert_eq!(instance.disk_capacity, 2361393152);
        assert_eq!(instance.ssh_port, 14357);
        assert!(instance.display);
        assert!(instance.gpu);
        assert!(instance.mounts.is_empty());
        assert!(instance.hostfwd.is_empty());
    }

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
            display: false,
            gpu: false,
            mounts: Vec::new(),
            hostfwd: Vec::new(),
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
  display: false
  gpu: false
  mounts: []
  hostfwd: []
"#
        );
    }
}
