use crate::actions::LoadInstanceAction;
use crate::commands::{self, Command};
use crate::error::Result;
use crate::util;
use crate::view::{Alignment, Console, TableView};
use clap::Parser;
use std::sync::Arc;

/// List VM instances
///
/// Examples:
///
///   $ cubic instances
///   Name          Arch    CPUs   Memory       Disk   Running
///   noble-arm64   arm64      8   8192 M    4/100 G       yes
///   trixie        amd64      6     16 G      100 G       yes
///   fedora        amd64      4   4096 M   10/100 G        no
///
///   Show the process id of each running VM instance:
///   $ cubic instances --all
///   PID    Name          Arch    CPUs   Memory       Disk   Running
///          noble-arm64   arm64      8   8192 M    4/100 G       yes
///   1059   trixie        amd64      6     16 G      100 G       yes
///          fedora        amd64      4   4096 M   10/100 G        no
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct ListInstanceCommand {
    #[clap(flatten)]
    pub all: commands::AllInfoArg,
}

impl Command for ListInstanceCommand {
    async fn run(&self, console: &Arc<Console>, context: &commands::Context) -> Result<u8> {
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
            .add("Disk", Alignment::Right)
            .add("Running", Alignment::Right);

        for instance_name in &instance_names {
            let instance = LoadInstanceAction::new().run(context, console, instance_name)?;
            if instance_store.is_stale(&instance) {
                continue;
            }

            let row = view.add_row();
            if self.all.value {
                let pid = instance_store
                    .get_pid(&instance)
                    .map(|pid| pid.to_string())
                    .unwrap_or_default();
                row.add(&pid, Alignment::Left);
            }
            let disk = match &instance.disk_used {
                Some(used) => format!(
                    "{}/{}",
                    used.to_value_in(&instance.disk_capacity),
                    instance.disk_capacity.to_size()
                ),
                None => instance.disk_capacity.to_size(),
            };
            row.add(instance_name, Alignment::Left)
                .add(&instance.arch.to_string(), Alignment::Left)
                .add(&instance.cpus.to_string(), Alignment::Right)
                .add(&instance.mem.to_size(), Alignment::Right)
                .add(&disk, Alignment::Right)
                .add(
                    util::to_yes_no(instance_store.is_running(&instance)),
                    Alignment::Right,
                );
        }
        view.print(console);
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::InstanceStoreMock;
    use crate::models::{Arch, DataSize, Environment, Instance, UserName};
    use crate::platform::{System, SystemMock};
    use std::str::FromStr;
    use std::sync::Arc;

    fn build_context(instances: Vec<Instance>) -> commands::Context {
        build_context_with_store(InstanceStoreMock::new(instances))
    }

    fn build_context_with_store(store: InstanceStoreMock) -> commands::Context {
        let env = Environment::new(
            UserName::from_str("cubic").unwrap(),
            String::new(),
            String::new(),
        );
        commands::Context::new(Arc::new(SystemMock::new()), env, Box::new(store))
    }

    fn build_instances() -> Vec<Instance> {
        vec![
            Instance {
                name: "test".to_string(),
                arch: Arch::AMD64,
                user: UserName::from_str("cubic").unwrap(),
                cpus: 1,
                mem: DataSize::new(1024),
                disk_used: Some(DataSize::new(512 * 1024)),
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

    #[tokio::test]
    async fn test_list_instance_command() {
        let system = SystemMock::new();
        let system = Arc::new(system);
        let console = &Console::new(Arc::clone(&system) as Arc<dyn System>);
        let context = build_context(build_instances());

        ListInstanceCommand { all: false.into() }
            .run(console, &context)
            .await
            .unwrap();

        assert_eq!(
            system.get_output(),
            "\
Name    Arch    CPUs   Memory         Disk   Running
test    amd64      1   1024 B   512/1024 K        no
test2   amd64      5      0 B       5000 B        no
"
        );
    }

    #[tokio::test]
    async fn test_list_instance_command_all_adds_the_pid_column() {
        let system = SystemMock::new();
        let system = Arc::new(system);
        let console = &Console::new(Arc::clone(&system) as Arc<dyn System>);
        let context = build_context(build_instances());

        ListInstanceCommand { all: true.into() }
            .run(console, &context)
            .await
            .unwrap();

        assert_eq!(
            system.get_output(),
            "\
PID   Name    Arch    CPUs   Memory         Disk   Running
      test    amd64      1   1024 B   512/1024 K        no
      test2   amd64      5      0 B       5000 B        no
"
        );
    }

    #[tokio::test]
    async fn test_list_instance_command_all_shows_the_pid_of_a_running_instance() {
        let system = SystemMock::new();
        let system = Arc::new(system);
        let console = &Console::new(Arc::clone(&system) as Arc<dyn System>);
        let context = build_context_with_store(
            InstanceStoreMock::new_with_running(build_instances(), &["test2"])
                .set_pid("test2", 1059),
        );

        ListInstanceCommand { all: true.into() }
            .run(console, &context)
            .await
            .unwrap();

        assert_eq!(
            system.get_output(),
            "\
PID    Name    Arch    CPUs   Memory         Disk   Running
       test    amd64      1   1024 B   512/1024 K        no
1059   test2   amd64      5      0 B       5000 B       yes
"
        );
    }

    #[tokio::test]
    async fn test_list_instance_command_hides_a_stale_instance() {
        let system = SystemMock::new();
        let system = Arc::new(system);
        let console = &Console::new(Arc::clone(&system) as Arc<dyn System>);
        let mut instances = build_instances();
        instances[0].auto_remove = true;
        instances[1].auto_remove = true;
        let context =
            build_context_with_store(InstanceStoreMock::new_with_running(instances, &["test2"]));

        ListInstanceCommand { all: false.into() }
            .run(console, &context)
            .await
            .unwrap();

        assert_eq!(
            system.get_output(),
            "\
Name    Arch    CPUs   Memory     Disk   Running
test2   amd64      5      0 B   5000 B       yes
"
        );
    }

    #[tokio::test]
    async fn test_list_instance_command_empty() {
        let system = SystemMock::new();
        let system = Arc::new(system);
        let console = &Console::new(Arc::clone(&system) as Arc<dyn System>);
        let context = build_context(Vec::new());

        ListInstanceCommand { all: false.into() }
            .run(console, &context)
            .await
            .unwrap();

        assert_eq!(
            system.get_output(),
            "Name   Arch   CPUs   Memory   Disk   Running\n"
        );
    }
}
