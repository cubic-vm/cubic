use crate::actions::LoadInstanceAction;
use crate::commands::{self, Command};
use crate::error::Result;
use crate::util;
use crate::view::{Alignment, Console, TableView};
use clap::Parser;

/// List VM instances
///
/// Examples:
///
///   $ cubic instances
///   Name          Arch    CPUs     Memory   Disk Used   Disk Total   Running
///   noble-arm64   arm64      8    8.0 GiB     4.4 GiB    100.0 GiB       yes
///   trixie        amd64      6   16.0 GiB         n/a    100.0 GiB       yes
///   fedora        amd64      4    4.0 GiB    10.0 GiB    100.0 GiB        no
///
///   Show the process id of each running VM instance:
///   $ cubic instances --all
///   PID    Name          Arch    CPUs     Memory   Disk Used   Disk Total   Running
///          noble-arm64   arm64      8    8.0 GiB     4.4 GiB    100.0 GiB       yes
///   1059   trixie        amd64      6   16.0 GiB         n/a    100.0 GiB       yes
///          fedora        amd64      4    4.0 GiB    10.0 GiB    100.0 GiB        no
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct ListInstanceCommand {
    #[clap(flatten)]
    pub all: commands::AllInfoArg,
}

impl Command for ListInstanceCommand {
    fn run(&self, console: &mut Console<'_>, context: &commands::Context) -> Result<()> {
        let instance_store = context.get_instance_store();
        let instance_names = instance_store.get_instances();

        let mut view = TableView::new();
        let header = view.add_row();
        if self.all.value {
            header.add("PID", Alignment::Left);
        }
        header
            .add("Name", Alignment::Left)
            .add("Arch", Alignment::Left)
            .add("CPUs", Alignment::Right)
            .add("Memory", Alignment::Right)
            .add("Disk Used", Alignment::Right)
            .add("Disk Total", Alignment::Right)
            .add("Running", Alignment::Right);

        for instance_name in &instance_names {
            let instance = LoadInstanceAction::new().run(context, console, instance_name)?;

            let row = view.add_row();
            if self.all.value {
                let pid = instance_store
                    .get_pid(&instance)
                    .map(|pid| pid.to_string())
                    .unwrap_or_default();
                row.add(&pid, Alignment::Left);
            }
            row.add(instance_name, Alignment::Left)
                .add(&instance.arch.to_string(), Alignment::Left)
                .add(&instance.cpus.to_string(), Alignment::Right)
                .add(&instance.mem.to_size(), Alignment::Right)
                .add(
                    &util::format_or_na(instance.disk_used.as_ref().map(|size| size.to_size())),
                    Alignment::Right,
                )
                .add(&instance.disk_capacity.to_size(), Alignment::Right)
                .add(
                    util::to_yes_no(instance_store.is_running(&instance)),
                    Alignment::Right,
                );
        }
        view.print(console);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::InstanceStoreMock;
    use crate::models::{Arch, DataSize, Environment, Instance, UserName};
    use crate::platform::SystemMock;
    use std::rc::Rc;
    use std::str::FromStr;

    fn build_context(instances: Vec<Instance>) -> commands::Context {
        build_context_with_store(InstanceStoreMock::new(instances))
    }

    fn build_context_with_store(store: InstanceStoreMock) -> commands::Context {
        let env = Environment::new(
            UserName::from_str("cubic").unwrap(),
            String::new(),
            String::new(),
        );
        commands::Context::new(Rc::new(SystemMock::new()), env, Box::new(store))
    }

    fn build_instances() -> Vec<Instance> {
        vec![
            Instance {
                name: "test".to_string(),
                arch: Arch::AMD64,
                user: UserName::from_str("cubic").unwrap(),
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
                user: UserName::from_str("cubic").unwrap(),
                cpus: 5,
                mem: DataSize::new(0),
                disk_capacity: DataSize::new(5000),
                ssh_port: 9000,
                hostfwd: Vec::new(),
                ..Instance::default()
            },
        ]
    }

    #[test]
    fn test_list_instance_command() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let context = build_context(build_instances());

        ListInstanceCommand { all: false.into() }
            .run(console, &context)
            .unwrap();

        assert_eq!(
            system.get_output(),
            "\
Name    Arch    CPUs    Memory   Disk Used   Disk Total   Running
test    amd64      1   1.0 KiB         n/a      1.0 MiB        no
test2   amd64      5     0   B         n/a      4.9 KiB        no
"
        );
    }

    #[test]
    fn test_list_instance_command_all_adds_the_pid_column() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let context = build_context(build_instances());

        ListInstanceCommand { all: true.into() }
            .run(console, &context)
            .unwrap();

        assert_eq!(
            system.get_output(),
            "\
PID   Name    Arch    CPUs    Memory   Disk Used   Disk Total   Running
      test    amd64      1   1.0 KiB         n/a      1.0 MiB        no
      test2   amd64      5     0   B         n/a      4.9 KiB        no
"
        );
    }

    #[test]
    fn test_list_instance_command_all_shows_the_pid_of_a_running_instance() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let context = build_context_with_store(
            InstanceStoreMock::new_with_running(build_instances(), &["test2"])
                .set_pid("test2", 1059),
        );

        ListInstanceCommand { all: true.into() }
            .run(console, &context)
            .unwrap();

        assert_eq!(
            system.get_output(),
            "\
PID    Name    Arch    CPUs    Memory   Disk Used   Disk Total   Running
       test    amd64      1   1.0 KiB         n/a      1.0 MiB        no
1059   test2   amd64      5     0   B         n/a      4.9 KiB       yes
"
        );
    }

    #[test]
    fn test_list_instance_command_empty() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let context = build_context(Vec::new());

        ListInstanceCommand { all: false.into() }
            .run(console, &context)
            .unwrap();

        assert_eq!(
            system.get_output(),
            "Name   Arch   CPUs   Memory   Disk Used   Disk Total   Running\n"
        );
    }
}
