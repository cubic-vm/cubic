use crate::error::Error;
use crate::instance::Instance;
use std::io::Write;

#[derive(Default)]
pub struct InstanceSerializer;

impl InstanceSerializer {
    pub fn new() -> Self {
        Self
    }

    pub fn serialize(&self, instance: &Instance, writer: &mut dyn Write) -> Result<(), Error> {
        toml::to_string(instance)
            .map(|content| writer.write_all(&content.into_bytes()))
            .map(|_| ())
            .map_err(Error::SerdeToml)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arch::Arch;

    #[test]
    fn test_serialize_minimal_config() {
        let mut writer = Vec::new();

        InstanceSerializer::new()
            .serialize(
                &Instance {
                    name: "test".to_string(),
                    arch: Arch::AMD64,
                    user: "tux".to_string(),
                    cpus: 1,
                    mem: 1000,
                    disk_capacity: 1000,
                    ssh_port: 10000,
                    hostfwd: Vec::new(),
                    ..Instance::default()
                },
                &mut writer,
            )
            .expect("Cannot parser config");
        let config = String::from_utf8(writer).unwrap();

        assert_eq!(
            config,
            r#"arch = "AMD64"
user = "tux"
cpus = 1
mem = 1000
disk_capacity = 1000
ssh_port = 10000
hostfwd = []
"#
        );
    }
}
