use crate::error::Error;
use crate::instance::{Instance, InstanceDeserializer};
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
    fn deserialize(&self, name: &str, reader: &mut dyn Read) -> Result<Instance, Error> {
        let mut data = String::new();
        reader.read_to_string(&mut data).map_err(Error::Io)?;
        toml::from_str(&data)
            .map(|mut instance: Instance| {
                instance.name = name.to_string();
                instance
            })
            .map_err(|_| Error::CannotParseFile(String::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::BufReader;

    #[test]
    fn test_deserialize_empty_file() {
        let reader = &mut BufReader::new("".as_bytes());
        let instance = TomlInstanceDeserializer::new().deserialize("test", reader);
        assert!(instance.is_err());
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
            .expect("Cannot parser config");
        assert_eq!(instance.name, "test");
        assert_eq!(instance.user, "cubic");
        assert_eq!(instance.cpus, 1);
        assert_eq!(instance.mem.get_bytes(), 1073741824);
        assert_eq!(instance.disk_capacity.get_bytes(), 2361393152);
        assert_eq!(instance.ssh_port, 14357);
        assert!(instance.hostfwd.is_empty());
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
"#
            .as_bytes(),
        );

        let instance = TomlInstanceDeserializer::new()
            .deserialize("test", reader)
            .expect("Cannot parser config");
        assert_eq!(instance.name, "test");
        assert_eq!(instance.user, "tux");
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
    }

    #[test]
    fn test_deserialize_desktop_config() {
        let reader = &mut BufReader::new(
            r#"
user = "tux"
cpus = 1
mem = 1073741824
disk_capacity = 2361393152
ssh_port = 14357
"#
            .as_bytes(),
        );

        let instance = TomlInstanceDeserializer::new()
            .deserialize("test", reader)
            .expect("Cannot parser config");
        assert_eq!(instance.name, "test");
        assert_eq!(instance.user, "tux");
        assert_eq!(instance.cpus, 1);
        assert_eq!(instance.mem.get_bytes(), 1073741824);
        assert_eq!(instance.disk_capacity.get_bytes(), 2361393152);
        assert_eq!(instance.ssh_port, 14357);
        assert!(instance.hostfwd.is_empty());
    }
}
