use crate::error::Error;
use crate::instance::InstanceStore;
use crate::util;
use crate::view::{Console, MapView};
use clap::Parser;

/// Get information about an virtual machine instance
#[derive(Parser)]
pub struct InstanceInfoCommand {
    /// Name of the virtual machine instance
    instance: String,
}

impl InstanceInfoCommand {
    pub fn run(
        &self,
        console: &mut dyn Console,
        instance_store: &dyn InstanceStore,
    ) -> Result<(), Error> {
        if !instance_store.exists(&self.instance) {
            return Result::Err(Error::UnknownInstance(self.instance.to_string()));
        }

        let instance = instance_store.load(&self.instance)?;

        let mut view = MapView::new();
        view.add("Arch", &instance.arch.to_string());
        view.add("CPUs", &instance.cpus.to_string());
        view.add("Memory", &util::bytes_to_human_readable(instance.mem));
        view.add(
            "Disk",
            &util::bytes_to_human_readable(instance.disk_capacity),
        );
        view.add("User", &instance.user);
        view.add("SSH Port", &instance.ssh_port.to_string());

        for (index, rule) in instance.hostfwd.iter().enumerate() {
            let key = if index == 0 { "Forward" } else { "" };
            view.add(key, rule);
        }

        view.print(console);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arch::Arch;
    use crate::instance::instance_store_mock::tests::InstanceStoreMock;
    use crate::instance::Instance;
    use crate::view::console_mock::tests::ConsoleMock;

    #[test]
    fn test_info_command() {
        let console = &mut ConsoleMock::new();
        let instance_store = &InstanceStoreMock::new(vec![Instance {
            name: "test".to_string(),
            arch: Arch::AMD64,
            user: "cubic".to_string(),
            cpus: 1,
            mem: 1024,
            disk_capacity: 1048576,
            ssh_port: 9000,
            hostfwd: Vec::new(),
        }]);

        InstanceInfoCommand {
            instance: "test".to_string(),
        }
        .run(console, instance_store)
        .unwrap();

        assert_eq!(
            console.get_output(),
            "\
Arch:     amd64
CPUs:     1
Memory:   1.0 KiB
Disk:     1.0 MiB
User:     cubic
SSH Port: 9000
"
        );
    }

    #[test]
    fn test_info_command_failed() {
        let console = &mut ConsoleMock::new();
        let instance_store = &InstanceStoreMock::new(Vec::new());

        assert!(matches!(
            InstanceInfoCommand {
                instance: "test".to_string()
            }
            .run(console, instance_store),
            Result::Err(Error::UnknownInstance(_))
        ));
    }
}
