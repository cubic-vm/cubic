use crate::error::Error;
use crate::instance::InstanceStore;
use crate::util;
use crate::view::{Console, MapView};

pub struct InstanceInfoCommand;

impl InstanceInfoCommand {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(
        &self,
        console: &mut dyn Console,
        instance_store: &dyn InstanceStore,
        instance: &str,
    ) -> Result<(), Error> {
        if !instance_store.exists(instance) {
            return Result::Err(Error::UnknownInstance(instance.to_string()));
        }

        let instance = instance_store.load(instance)?;

        let mut view = MapView::new();
        view.add("Arch", &instance.arch.to_string());
        view.add("CPUs", &instance.cpus.to_string());
        view.add("Memory", &util::bytes_to_human_readable(instance.mem));
        view.add(
            "Disk",
            &util::bytes_to_human_readable(instance.disk_capacity),
        );
        view.add("User", &instance.user);
        view.add("Display", &instance.display.to_string());
        view.add("GPU", &instance.gpu.to_string());
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
            display: false,
            gpu: false,
            hostfwd: Vec::new(),
        }]);

        InstanceInfoCommand::new()
            .run(console, instance_store, "test")
            .unwrap();

        assert_eq!(
            console.get_output(),
            "\
Arch:     amd64
CPUs:     1
Memory:   1.0 KiB
Disk:     1.0 MiB
User:     cubic
Display:  false
GPU:      false
SSH Port: 9000
"
        );
    }

    #[test]
    fn test_info_command_failed() {
        let console = &mut ConsoleMock::new();
        let instance_store = &InstanceStoreMock::new(Vec::new());

        assert!(matches!(
            InstanceInfoCommand::new().run(console, instance_store, "test"),
            Result::Err(Error::UnknownInstance(_))
        ));
    }
}
