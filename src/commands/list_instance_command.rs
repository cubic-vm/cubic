use crate::commands::Command;
use crate::env::Environment;
use crate::error::Error;
use crate::image::ImageStore;
use crate::instance::InstanceStore;
use crate::view::{Alignment, Console, TableView};
use clap::Parser;

/// List all virtual machine instances
#[derive(Parser)]
pub struct ListInstanceCommand;

impl Command for ListInstanceCommand {
    fn run(
        &self,
        console: &mut dyn Console,
        _env: &Environment,
        _image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<(), Error> {
        let instance_names = instance_store.get_instances();

        let mut view = TableView::new();
        view.add_row()
            .add("PID", Alignment::Left)
            .add("Name", Alignment::Left)
            .add("Arch", Alignment::Left)
            .add("CPUs", Alignment::Right)
            .add("Memory", Alignment::Right)
            .add("Disk Used", Alignment::Right)
            .add("Disk Total", Alignment::Right)
            .add("State", Alignment::Left);

        for instance_name in &instance_names {
            let instance = instance_store.load(instance_name)?;
            let pid = instance_store
                .get_pid(&instance)
                .map(|pid| pid.to_string())
                .unwrap_or_default();

            view.add_row()
                .add(&pid, Alignment::Left)
                .add(instance_name, Alignment::Left)
                .add(&instance.arch.to_string(), Alignment::Left)
                .add(&instance.cpus.to_string(), Alignment::Right)
                .add(&instance.mem.to_size(), Alignment::Right)
                .add(
                    &instance
                        .disk_used
                        .as_ref()
                        .map(|size| size.to_size())
                        .unwrap_or("n/a".to_string()),
                    Alignment::Right,
                )
                .add(&instance.disk_capacity.to_size(), Alignment::Right)
                .add(
                    if instance_store.is_running(&instance) {
                        "running"
                    } else {
                        "stopped"
                    },
                    Alignment::Left,
                );
        }
        view.print(console);
        Result::Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arch::Arch;
    use crate::image::image_store_mock::tests::ImageStoreMock;
    use crate::instance::Instance;
    use crate::instance::instance_store_mock::tests::InstanceStoreMock;
    use crate::model::DataSize;
    use crate::view::console_mock::tests::ConsoleMock;

    #[test]
    fn test_list_instance_command() {
        let console = &mut ConsoleMock::new();
        let image_store = &ImageStoreMock::default();
        let env = &Environment::new(String::new(), String::new(), String::new());
        let instance_store = &InstanceStoreMock::new(vec![
            Instance {
                name: "test".to_string(),
                arch: Arch::AMD64,
                user: "cubic".to_string(),
                cpus: 1,
                mem: DataSize::new(1024),
                disk_capacity: DataSize::new(1048576),
                ssh_port: 9000,
                hostfwd: Vec::new(),
                ..Instance::default()
            },
            Instance {
                name: "test2".to_string(),
                arch: Arch::AMD64,
                user: "cubic".to_string(),
                cpus: 5,
                mem: DataSize::new(0),
                disk_capacity: DataSize::new(5000),
                ssh_port: 9000,
                hostfwd: Vec::new(),
                ..Instance::default()
            },
        ]);

        ListInstanceCommand {}
            .run(console, env, image_store, instance_store)
            .unwrap();

        assert_eq!(
            console.get_output(),
            "\
PID   Name    Arch    CPUs    Memory   Disk Used   Disk Total   State
      test    amd64      1   1.0 KiB         n/a      1.0 MiB   stopped
      test2   amd64      5     0   B         n/a      4.9 KiB   stopped
"
        );
    }

    #[test]
    fn test_list_instance_command_empty() {
        let console = &mut ConsoleMock::new();
        let instance_store = &InstanceStoreMock::new(Vec::new());
        let image_store = &ImageStoreMock::default();
        let env = &Environment::new(String::new(), String::new(), String::new());

        ListInstanceCommand {}
            .run(console, env, image_store, instance_store)
            .unwrap();

        assert_eq!(
            console.get_output(),
            "PID   Name   Arch   CPUs   Memory   Disk Used   Disk Total   State\n"
        );
    }
}
