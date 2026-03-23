use crate::commands::Command;
use crate::env::Environment;
use crate::error::Result;
use crate::image::ImageStore;
use crate::instance::InstanceStore;
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
    fn run(
        &self,
        console: &mut dyn Console,
        _env: &Environment,
        _image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<()> {
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
            let status = if instance_store.is_running(instance) {
                "yes"
            } else {
                "no"
            };

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
