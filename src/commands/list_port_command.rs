use crate::actions::LoadInstanceAction;
use crate::commands::{self, Command};
use crate::error::Result;
use crate::util;
use crate::view::{Alignment, Console, TableView};
use clap::Parser;

/// List ports for VM instances
///
/// Shows port forwarding rules from VM instance to host. Use cubic modify <instance>
/// to configure the forwarding. The SSH port is assigned by cubic and is shown by
/// cubic show instead.
///
/// Examples:
///
///   $ cubic ports
///   Instance      Host             Guest   Protocol   In Use
///   noble-arm64   127.0.0.1:2222   :22     /tcp       no
///   trixie        127.0.0.1:4000   :4000   /tcp       yes
///   trixie        0.0.0.0:80       :8000   /udp       yes
///
///   $ cubic ports
///   No port forwarding rules are configured.
///   Add one with cubic modify <instance> --port <host_port>:<guest_port>
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct ListPortCommand;

impl Command for ListPortCommand {
    fn run(&self, console: &mut Console<'_>, context: &commands::Context) -> Result<()> {
        let instance_store = context.get_instance_store();
        let instance_names = instance_store.get_instances();

        let mut rule_count = 0;
        let mut view = TableView::new();
        view.add_row()
            .add("Instance", Alignment::Left)
            .add("Host", Alignment::Left)
            .add("Guest", Alignment::Left)
            .add("Protocol", Alignment::Left)
            .add("In Use", Alignment::Left);

        for instance_name in instance_names {
            let instance = &LoadInstanceAction::new().run(context, console, &instance_name)?;
            if instance.hostfwd.is_empty() {
                continue;
            }

            let status = util::to_yes_no(instance_store.is_running(instance));
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
                rule_count += 1;
            }
        }

        if rule_count == 0 {
            console.print("No port forwarding rules are configured.");
            console.print("Add one with cubic modify <instance> --port <host_port>:<guest_port>");
            return Ok(());
        }

        view.print(console);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::InstanceStoreMock;
    use crate::models::{Environment, Instance, UserName};
    use crate::platform::SystemMock;
    use std::rc::Rc;
    use std::str::FromStr;

    fn build_context(instances: Vec<Instance>) -> commands::Context {
        let env = Environment::new(
            UserName::from_str("cubic").unwrap(),
            String::new(),
            String::new(),
            String::new(),
        );
        commands::Context::new(
            Rc::new(SystemMock::new()),
            env,
            Box::new(InstanceStoreMock::new(instances)),
        )
    }

    const NO_RULES: &str = "\
No port forwarding rules are configured.
Add one with cubic modify <instance> --port <host_port>:<guest_port>
";

    #[test]
    fn test_list_ports_without_instances_explains_how_to_add_a_rule() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let context = build_context(Vec::new());

        ListPortCommand {}.run(console, &context).unwrap();

        assert_eq!(system.get_output(), NO_RULES);
    }

    #[test]
    fn test_list_ports_without_rules_explains_how_to_add_a_rule() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let context = build_context(vec![Instance {
            name: "test".to_string(),
            ssh_port: 9000,
            ..Instance::default()
        }]);

        ListPortCommand {}.run(console, &context).unwrap();

        assert_eq!(system.get_output(), NO_RULES);
    }

    #[test]
    fn test_list_ports_skips_instances_without_rules() {
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
test2      127.0.0.1:4000   :40     /tcp       no
"
        );
    }
}
