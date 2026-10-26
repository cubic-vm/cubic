use crate::error::{Error, Result};
use crate::models::Instance;

#[derive(Default)]
pub struct InstanceSerializer;

impl InstanceSerializer {
    pub fn new() -> Self {
        Self
    }

    pub fn serialize(&self, instance: &Instance) -> Result<String> {
        toml::to_string(instance).map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Arch, DataSize, UserName};
    use std::str::FromStr;

    #[test]
    fn test_serialize_minimal_config() {
        let config = InstanceSerializer::new()
            .serialize(&Instance {
                name: "test".to_string(),
                arch: Arch::AMD64,
                user: UserName::from_str("tux").unwrap(),
                cpus: 1,
                mem: DataSize::new(1000),
                disk_capacity: DataSize::new(1000),
                ssh_port: 10000,
                hostfwd: Vec::new(),
                execute: None,
                ..Instance::default()
            })
            .expect("Cannot parser config");

        assert_eq!(
            config,
            r#"arch = "AMD64"
user = "tux"
cpus = 1
mem = 1000
disk_capacity = 1000
ssh_port = 10000
hostfwd = []
isolate = false
auto_remove = false
first_boot = false
"#
        );
    }

    #[test]
    fn test_serialize_full_config() {
        let config = InstanceSerializer::new()
            .serialize(&Instance {
                name: "test".to_string(),
                arch: Arch::AMD64,
                user: UserName::from_str("tux").unwrap(),
                os: Some("debian:12".to_string()),
                cpus: 1,
                mem: DataSize::new(1000),
                disk_capacity: DataSize::new(1000),
                ssh_port: 10000,
                hostfwd: Vec::new(),
                execute: Some("echo hello world".to_string()),
                isolate: true,
                ssh_host_key: Some("ssh-ed25519 AAAA".to_string()),
                first_boot: true,
                ..Instance::default()
            })
            .expect("Cannot parser config");

        assert_eq!(
            config,
            r#"arch = "AMD64"
user = "tux"
os = "debian:12"
cpus = 1
mem = 1000
disk_capacity = 1000
ssh_port = 10000
hostfwd = []
execute = "echo hello world"
isolate = true
ssh_host_key = "ssh-ed25519 AAAA"
auto_remove = false
first_boot = true
"#
        );
    }
}
