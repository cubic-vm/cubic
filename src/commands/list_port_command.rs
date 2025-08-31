use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore};
use crate::view::{Alignment, Console, TableView};
use clap::Parser;

/// List forwarded ports for all virtual machine instances
#[derive(Parser)]
pub struct ListPortCommand;

impl ListPortCommand {
    pub fn run(&self, console: &mut dyn Console, instance_dao: &InstanceDao) -> Result<(), Error> {
        let instance_names = instance_dao.get_instances();

        let mut view = TableView::new();
        view.add_row()
            .add("Instance", Alignment::Left)
            .add("Host", Alignment::Left)
            .add("Guest", Alignment::Left)
            .add("Protocol", Alignment::Left)
            .add("State", Alignment::Left);

        for instance_name in instance_names {
            let instance = &instance_dao.load(&instance_name)?;
            for rule in &instance.hostfwd {
                view.add_row()
                    .add(&instance_name, Alignment::Left)
                    .add(
                        &format!("{}:{}", rule.get_host_ip(), rule.get_host_port()),
                        Alignment::Left,
                    )
                    .add(&format!(":{}", rule.get_guest_port()), Alignment::Left)
                    .add(&format!("/{}", rule.get_protocol()), Alignment::Left)
                    .add(
                        instance_dao
                            .is_running(instance)
                            .then_some("in use")
                            .unwrap_or_default(),
                        Alignment::Left,
                    );
            }
        }
        view.print(console);
        Ok(())
    }
}
