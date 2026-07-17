use crate::commands::{self, Command};
use crate::error::Result;
use crate::util;
use crate::view::{Alignment, Console, TableView};
use clap::Parser;

/// List ports for VM instances
///
/// Shows port forwarding rules from VM instance to host. Use cubic modify <instance>
/// to configure the forwarding.
///
/// Examples:
///
///   $ cubic ports
///   Instance       Host              Guest   Protocol   In Use
///   noble-arm64    127.0.0.1:30612   :22     /tcp       no
///   noble-arm64    127.0.0.1:2222    :22     /tcp       no
///   trixie         127.0.0.1:30200   :22     /tcp       yes
///   trixie         127.0.0.1:4000    :4000   /tcp       yes
///   fedora         127.0.0.1:59153   :22     /tcp       no
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct ListPortCommand;

impl Command for ListPortCommand {
    fn run(&self, console: &mut Console<'_>, context: &commands::Context) -> Result<()> {
        let instance_store = context.get_instance_store();
        let instance_names = instance_store.get_instances();

        let mut view = TableView::new();
        view.add_row()
            .add("Instance", Alignment::Left)
            .add("Host", Alignment::Left)
            .add("Guest", Alignment::Left)
            .add("Protocol", Alignment::Left)
            .add("In Use", Alignment::Left);

        for instance_name in instance_names {
            let instance = &instance_store.load(&instance_name)?;
            let status = util::to_yes_no(instance_store.is_running(instance));

            view.add_row()
                .add(&instance_name, Alignment::Left)
                .add(&format!("127.0.0.1:{}", instance.ssh_port), Alignment::Left)
                .add(":22", Alignment::Left)
                .add("/tcp", Alignment::Left)
                .add(status, Alignment::Left);

            for rule in &instance.hostfwd {
                view.add_row()
                    .add(&instance_name, Alignment::Left)
                    .add(
                        &format!("{}:{}", rule.get_host_ip(), rule.get_host_port()),
                        Alignment::Left,
                    )
                    .add(&format!(":{}", rule.get_guest_port()), Alignment::Left)
                    .add(&format!("/{}", rule.get_protocol()), Alignment::Left)
                    .add(status, Alignment::Left);
            }
        }
        view.print(console);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::ImageStoreMock;
    use crate::instance::InstanceStoreMock;
    use crate::models::{Environment, Instance};
    use crate::platform::SystemMock;
    use std::rc::Rc;

    fn build_context(instances: Vec<Instance>) -> commands::Context {
        let env = Environment::new(
            "cubic".to_string(),
            String::new(),
            String::new(),
            String::new(),
        );
        commands::Context::new(
            Rc::new(SystemMock::new()),
            env,
            Box::new(ImageStoreMock::default()),
            Box::new(InstanceStoreMock::new(instances)),
        )
    }

    #[test]
    fn test_list_ports_empty() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let context = build_context(Vec::new());

        ListPortCommand {}.run(console, &context).unwrap();

        assert_eq!(
            system.get_output(),
            "Instance   Host   Guest   Protocol   In Use\n"
        );
    }

    #[test]
    fn test_list_ports_adds_row_per_forward_rule() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let context = build_context(vec![
            Instance {
                name: "test".to_string(),
                ssh_port: 9000,
                ..Instance::default()
            },
            Instance {
                name: "test2".to_string(),
                ssh_port: 8000,
                hostfwd: vec!["127.0.0.1:4000:40/tcp".parse().unwrap()],
                ..Instance::default()
            },
        ]);

        ListPortCommand {}.run(console, &context).unwrap();

        assert_eq!(
            system.get_output(),
            "\
Instance   Host             Guest   Protocol   In Use
test       127.0.0.1:9000   :22     /tcp       no
test2      127.0.0.1:8000   :22     /tcp       no
test2      127.0.0.1:4000   :40     /tcp       no
"
        );
    }
}
