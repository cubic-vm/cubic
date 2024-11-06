pub mod machine_dao;

pub use crate::error::Error;
pub use machine_dao::*;
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

impl Machine {
    pub fn deserialize(name: &str, reader: &mut dyn Read) -> Result<Machine, Error> {
        serde_yaml::from_reader(reader)
            .map(|config: Config| config.machine)
            .map(|mut machine: Machine| {
                machine.name = name.to_string();
                machine
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
        .map_err(|_| Error::CannotWriteFile(String::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::BufReader;

    #[test]
    fn test_deserialize_empty_file() {
        let reader = &mut BufReader::new("".as_bytes());
        let machine = Machine::deserialize("test", reader);
        assert!(machine.is_err());
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

        let machine = Machine::deserialize("test", reader).expect("Cannot parser config");
        assert_eq!(machine.name, "test");
        assert_eq!(machine.user, "cubic");
        assert_eq!(machine.cpus, 1);
        assert_eq!(machine.mem, 1073741824);
        assert_eq!(machine.disk_capacity, 2361393152);
        assert_eq!(machine.ssh_port, 14357);
        assert!(!machine.display);
        assert!(!machine.gpu);
        assert!(machine.mounts.is_empty());
        assert!(machine.hostfwd.is_empty());
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

        let machine = Machine::deserialize("test", reader).expect("Cannot parser config");
        assert_eq!(machine.name, "test");
        assert_eq!(machine.user, "tux");
        assert_eq!(machine.cpus, 1);
        assert_eq!(machine.mem, 1073741824);
        assert_eq!(machine.disk_capacity, 2361393152);
        assert_eq!(machine.ssh_port, 14357);
        assert!(!machine.display);
        assert!(!machine.gpu);
        assert_eq!(
            machine.mounts,
            [MountPoint {
                host: "/home/tux/guest".to_string(),
                guest: "/home/tux".to_string()
            }]
        );
        assert_eq!(
            machine.hostfwd,
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

        let machine = Machine::deserialize("test", reader).expect("Cannot parser config");
        assert_eq!(machine.name, "test");
        assert_eq!(machine.user, "tux");
        assert_eq!(machine.cpus, 1);
        assert_eq!(machine.mem, 1073741824);
        assert_eq!(machine.disk_capacity, 2361393152);
        assert_eq!(machine.ssh_port, 14357);
        assert!(machine.display);
        assert!(machine.gpu);
        assert!(machine.mounts.is_empty());
        assert!(machine.hostfwd.is_empty());
    }

    #[test]
    fn test_serialize_minimal_config() {
        let mut writer = Vec::new();

        Machine {
            name: "test".to_string(),
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
