use crate::commands::Command;
use crate::env::Environment;
use crate::error::Error;
use crate::image::ImageStore;
use crate::instance::InstanceStore;
use crate::view::{Alignment, Console, TableView};
use clap::Parser;

/// List forwarded ports for all virtual machine instances
#[derive(Parser)]
pub struct ListPortCommand;

impl Command for ListPortCommand {
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
            .add("Instance", Alignment::Left)
            .add("Host", Alignment::Left)
            .add("Guest", Alignment::Left)
            .add("Protocol", Alignment::Left)
            .add("State", Alignment::Left);

        for instance_name in instance_names {
            let instance = &instance_store.load(&instance_name)?;
            let status = instance_store
                .is_running(instance)
                .then_some("in use")
                .unwrap_or_default();

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
