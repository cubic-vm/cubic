use crate::instance::InstanceDeserializer;
use crate::models::Instance;
use std::io::Read;
use std::str;

#[derive(Default)]
pub struct TomlInstanceDeserializer;

impl TomlInstanceDeserializer {
    pub fn new() -> Self {
        Self
    }
}

impl InstanceDeserializer for TomlInstanceDeserializer {
    fn deserialize(&self, name: &str, reader: &mut dyn Read) -> Option<Instance> {
        let mut data = String::new();
        reader.read_to_string(&mut data).ok()?;
        toml::from_str(&data)
            .map(|mut instance: Instance| {
                instance.name = name.to_string();
                instance
            })
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::BufReader;

    #[test]
    fn test_deserialize_empty_file() {
        let reader = &mut BufReader::new("".as_bytes());
        assert_eq!(
            TomlInstanceDeserializer::new().deserialize("test", reader),
            None
        );
    }

    #[test]
    fn test_deserialize_minimal_config() {
        let reader = &mut BufReader::new(
            r#"
cpus = 1
mem = 1073741824
disk_capacity = 2361393152
ssh_port = 14357
"#
            .as_bytes(),
        );

        let instance = TomlInstanceDeserializer::new()
            .deserialize("test", reader)
            .expect("Cannot parse config");
        assert_eq!(instance.name, "test");
        assert_eq!(instance.user.as_str(), "cubic");
        assert_eq!(instance.cpus, 1);
        assert_eq!(instance.mem.get_bytes(), 1073741824);
        assert_eq!(instance.disk_capacity.get_bytes(), 2361393152);
        assert_eq!(instance.ssh_port, 14357);
        assert!(instance.hostfwd.is_empty());
        assert_eq!(instance.execute, None);
        assert!(!instance.isolate);
    }

    #[test]
    fn test_deserialize_full_config() {
        let reader = &mut BufReader::new(
            r#"
user = "tux"
cpus = 1
mem = 1073741824
disk_capacity = 2361393152
ssh_port = 14357
hostfwd = ["tcp:127.0.0.1:8000-:8000", "tcp:127.0.0.1:9000-:10000"]
execute = "sudo apt update"
isolate = true
"#
            .as_bytes(),
        );

        let instance = TomlInstanceDeserializer::new()
            .deserialize("test", reader)
            .expect("Cannot parse config");
        assert_eq!(instance.name, "test");
        assert_eq!(instance.user.as_str(), "tux");
        assert_eq!(instance.cpus, 1);
        assert_eq!(instance.mem.get_bytes(), 1073741824);
        assert_eq!(instance.disk_capacity.get_bytes(), 2361393152);
        assert_eq!(instance.ssh_port, 14357);
        assert_eq!(
            instance
                .hostfwd
                .iter()
                .map(|rule| rule.to_qemu())
                .collect::<Vec<_>>(),
            ["tcp:127.0.0.1:8000-:8000", "tcp:127.0.0.1:9000-:10000"]
        );
        assert_eq!(instance.execute, Some("sudo apt update".to_string()));
        assert!(instance.isolate);
    }

    #[test]
    fn test_deserialize_falls_back_to_default_user_instead_of_discarding_config() {
        let reader = &mut BufReader::new(
            r#"
user = "Bad Name"
cpus = 4
mem = 1073741824
disk_capacity = 2361393152
ssh_port = 14357
"#
            .as_bytes(),
        );

        let instance = TomlInstanceDeserializer::new()
            .deserialize("test", reader)
            .expect("Cannot parse config");
        assert_eq!(instance.name, "test");
        assert_eq!(instance.user.as_str(), "cubic");
        assert_eq!(instance.cpus, 4);
        assert_eq!(instance.ssh_port, 14357);
    }
}
