use crate::actions::LoadInstanceAction;
use crate::commands::{self, Command};
use crate::error::Result;
use crate::util;
use crate::view::{Alignment, TableView};
use clap::Parser;

/// List VM instances
///
/// Examples:
///
///   $ cubic instances
///   Name          OS             Arch    vCPUs   Memory       Disk   Running
///   noble-arm64   ubuntu:24.04   arm64       8   8192 M    4/100 G       yes
///   trixie        debian:13      amd64       6     16 G      100 G       yes
///   fedora        fedora:42      amd64       4   4096 M   10/100 G        no
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct ListInstanceCommand {
    // Show all information
    #[clap(flatten)]
    pub all: commands::AllInfoArg,
}

impl Command for ListInstanceCommand {
    async fn run(&self, context: &commands::Context) -> Result<u8> {
        let instance_store = context.get_instance_store();
        let instance_names = instance_store.get_instances();

        let mut view = TableView::new();
        view.add_row()
            .add("Name", Alignment::Left)
            .add("OS", Alignment::Left)
            .add("Arch", Alignment::Left)
            .add("vCPUs", Alignment::Right)
            .add("Memory", Alignment::Right)
            .add("Disk", Alignment::Right)
            .add("Running", Alignment::Right);

        for instance_name in &instance_names {
            let instance = LoadInstanceAction::new().run(context, instance_name)?;
            if context.is_stale(&instance) {
                continue;
            }

            let row = view.add_row();
            let disk = match &instance.disk_used {
                Some(used) => format!(
                    "{}/{}",
                    used.to_value_in(&instance.disk_capacity),
                    instance.disk_capacity.to_size()
                ),
                None => instance.disk_capacity.to_size(),
            };
            let running = util::to_yes_no(context.get_monitor(&instance).is_running());
            row.add(instance_name, Alignment::Left)
                .add(instance.os.as_deref().unwrap_or_default(), Alignment::Left)
                .add(&instance.arch.to_string(), Alignment::Left)
                .add(&instance.cpus.to_string(), Alignment::Right)
                .add(&instance.mem.to_size(), Alignment::Right)
                .add(&disk, Alignment::Right)
                .add(running, Alignment::Right);
        }
        view.print(context.get_console());
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::ContextBuilder;
    use crate::models::{Arch, DataSize, Instance, UserName};
    use crate::platform::SystemMock;
    use std::str::FromStr;
    use std::sync::Arc;

    fn build_context(instances: Vec<Instance>, running: &[&str]) -> ContextBuilder {
        let mut builder = ContextBuilder::new(SystemMock::new());
        builder.get_store().add_instances(instances);
        builder.set_running(running);
        builder
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
                os: Some("ubuntu:24.04".to_string()),
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
        let builder = build_context(build_instances(), &[]);
        let system = Arc::clone(builder.get_system());
        let context = builder.build();

        ListInstanceCommand { all: false.into() }
            .run(&context)
            .await
            .unwrap();

        assert_eq!(
            system.get_output(),
            "\
Name    OS             Arch    vCPUs   Memory         Disk   Running
test    ubuntu:24.04   amd64       1   1024 B   512/1024 K        no
test2                  amd64       5      0 B       5000 B        no
"
        );
    }

    #[tokio::test]
    async fn test_list_instance_command_hides_a_stale_instance() {
        let mut instances = build_instances();
        instances[0].auto_remove = true;
        instances[1].auto_remove = true;
        let builder = build_context(instances, &["test2"]);
        let system = Arc::clone(builder.get_system());
        let context = builder.build();

        ListInstanceCommand { all: false.into() }
            .run(&context)
            .await
            .unwrap();

        assert_eq!(
            system.get_output(),
            "\
Name    OS   Arch    vCPUs   Memory     Disk   Running
test2        amd64       5      0 B   5000 B       yes
"
        );
    }
}
