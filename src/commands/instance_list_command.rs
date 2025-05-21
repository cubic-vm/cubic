use crate::error::Error;
use crate::instance::{InstanceState, InstanceStore};
use crate::util;
use crate::view::{Alignment, Console, TableView};

pub struct InstanceListCommand;

impl InstanceListCommand {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(
        &self,
        console: &mut dyn Console,
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
            .add("Disk", Alignment::Right)
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
                .add(
                    &util::bytes_to_human_readable(instance.mem),
                    Alignment::Right,
                )
                .add(
                    &util::bytes_to_human_readable(instance.disk_capacity),
                    Alignment::Right,
                )
                .add(
                    match instance_store.get_state(&instance) {
                        InstanceState::Stopped => "STOPPED",
                        InstanceState::Starting => "STARTING",
                        InstanceState::Running => "RUNNING",
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
    use crate::instance::instance_store_mock::tests::InstanceStoreMock;
    use crate::instance::Instance;
    use crate::view::console_mock::tests::ConsoleMock;

    #[test]
    fn test_instance_list_command() {
        let console = &mut ConsoleMock::new();
        let instance_store = &InstanceStoreMock::new(vec![
            Instance {
                name: "test".to_string(),
                arch: Arch::AMD64,
                user: "cubic".to_string(),
                cpus: 1,
                mem: 1024,
                disk_capacity: 1048576,
                ssh_port: 9000,
                display: false,
                gpu: false,
                mounts: Vec::new(),
                hostfwd: Vec::new(),
            },
            Instance {
                name: "test2".to_string(),
                arch: Arch::AMD64,
                user: "cubic".to_string(),
                cpus: 5,
                mem: 0,
                disk_capacity: 5000,
                ssh_port: 9000,
                display: false,
                gpu: false,
                mounts: Vec::new(),
                hostfwd: Vec::new(),
            },
        ]);

        InstanceListCommand::new()
            .run(console, instance_store)
            .unwrap();

        assert_eq!(
            console.get_output(),
            "\
PID   Name    Arch    CPUs    Memory      Disk   State
      test    amd64      1   1.0 KiB   1.0 MiB   STOPPED
      test2   amd64      5   0.0   B   4.9 KiB   STOPPED
"
        );
    }

    #[test]
    fn test_instance_list_command_empty() {
        let console = &mut ConsoleMock::new();
        let instance_store = &InstanceStoreMock::new(Vec::new());

        InstanceListCommand::new()
            .run(console, instance_store)
            .unwrap();

        assert_eq!(
            console.get_output(),
            "PID   Name   Arch   CPUs   Memory   Disk   State\n"
        );
    }
}
