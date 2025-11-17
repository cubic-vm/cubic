use crate::commands::Command;
use crate::env::Environment;
use crate::error::Error;
use crate::image::ImageStore;
use crate::instance::{InstanceName, InstanceStore};
use crate::model::DataSize;
use crate::view::{Console, MapView};
use clap::Parser;

/// Show a virtual machine instance
#[derive(Parser)]
pub struct InstanceShowCommand {
    /// Name of the virtual machine instance
    pub instance: InstanceName,
}

impl Command for InstanceShowCommand {
    fn run(
        &self,
        console: &mut dyn Console,
        _env: &Environment,
        _image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<(), Error> {
        if !instance_store.exists(self.instance.as_str()) {
            return Result::Err(Error::UnknownInstance(self.instance.to_string()));
        }

        let instance = instance_store.load(self.instance.as_str())?;

        let mut view = MapView::new();
        view.add("Arch", &instance.arch.to_string());
        view.add("CPUs", &instance.cpus.to_string());
        view.add("Memory", &DataSize::new(instance.mem as usize).to_size());
        view.add(
            "Disk Used",
            &instance
                .disk_used
                .map(|size| DataSize::new(size as usize).to_size())
                .unwrap_or("n/a".to_string()),
        );
        view.add(
            "Disk Total",
            &DataSize::new(instance.disk_capacity as usize).to_size(),
        );
        view.add("User", &instance.user);
        view.add("SSH Port", &instance.ssh_port.to_string());
        view.add(
            "SSH",
            &format!("ssh -p {} {}@localhost", instance.ssh_port, instance.user),
        );

        for (index, rule) in instance.hostfwd.iter().enumerate() {
            let key = if index == 0 { "Forward" } else { "" };
            view.add(key, &rule.to_string());
        }

        view.print(console);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arch::Arch;
    use crate::image::image_store_mock::tests::ImageStoreMock;
    use crate::instance::Instance;
    use crate::instance::instance_store_mock::tests::InstanceStoreMock;
    use crate::view::console_mock::tests::ConsoleMock;
    use std::str::FromStr;

    #[test]
    fn test_show_command1() {
        let console = &mut ConsoleMock::new();
        let env = &Environment::new(String::new(), String::new(), String::new());
        let image_store = &ImageStoreMock::default();
        let instance_store = &InstanceStoreMock::new(vec![Instance {
            name: "test".to_string(),
            arch: Arch::AMD64,
            user: "cubic".to_string(),
            cpus: 1,
            mem: 1024,
            disk_capacity: 1048576,
            ssh_port: 9000,
            hostfwd: Vec::new(),
            ..Instance::default()
        }]);

        InstanceShowCommand {
            instance: InstanceName::from_str("test").unwrap(),
        }
        .run(console, env, image_store, instance_store)
        .unwrap();

        assert_eq!(
            console.get_output(),
            "\
Arch:       amd64
CPUs:       1
Memory:     1.0 KiB
Disk Used:  n/a
Disk Total: 1.0 MiB
User:       cubic
SSH Port:   9000
SSH:        ssh -p 9000 cubic@localhost
"
        );
    }

    #[test]
    fn test_show_command2() {
        let console = &mut ConsoleMock::new();
        let env = &Environment::new(String::new(), String::new(), String::new());
        let image_store = &ImageStoreMock::default();
        let instance_store = &InstanceStoreMock::new(vec![Instance {
            name: "test".to_string(),
            arch: Arch::ARM64,
            user: "john".to_string(),
            cpus: 2,
            mem: 1,
            disk_capacity: 1,
            ssh_port: 8000,
            hostfwd: Vec::new(),
            ..Instance::default()
        }]);

        InstanceShowCommand {
            instance: InstanceName::from_str("test").unwrap(),
        }
        .run(console, env, image_store, instance_store)
        .unwrap();

        assert_eq!(
            console.get_output(),
            "\
Arch:       arm64
CPUs:       2
Memory:     1   B
Disk Used:  n/a
Disk Total: 1   B
User:       john
SSH Port:   8000
SSH:        ssh -p 8000 john@localhost
"
        );
    }

    #[test]
    fn test_show_command_failed() {
        let console = &mut ConsoleMock::new();
        let env = &Environment::new(String::new(), String::new(), String::new());
        let instance_store = &InstanceStoreMock::new(Vec::new());
        let image_store = &ImageStoreMock::default();

        assert!(matches!(
            InstanceShowCommand {
                instance: InstanceName::from_str("test").unwrap()
            }
            .run(console, env, image_store, instance_store),
            Result::Err(Error::UnknownInstance(_))
        ));
    }
}
