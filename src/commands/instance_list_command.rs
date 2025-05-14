use crate::error::Error;
use crate::instance::{InstanceDao, InstanceState};
use crate::util;
use crate::view::{Alignment, TableView};

pub struct InstanceListCommand;

impl InstanceListCommand {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self, instance_dao: &InstanceDao) -> Result<(), Error> {
        let instance_names = instance_dao.get_instances();

        let mut view = TableView::new();
        view.add_row()
            .add("PID", Alignment::Left)
            .add("Name", Alignment::Left)
            .add("CPUs", Alignment::Right)
            .add("Memory", Alignment::Right)
            .add("Disk", Alignment::Right)
            .add("State", Alignment::Left);

        for instance_name in &instance_names {
            let instance = instance_dao.load(instance_name)?;
            let pid = instance_dao
                .get_pid(&instance)
                .map(|pid| pid.to_string())
                .unwrap_or_default();

            view.add_row()
                .add(&pid, Alignment::Left)
                .add(instance_name, Alignment::Left)
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
                    match instance_dao.get_state(&instance) {
                        InstanceState::Stopped => "STOPPED",
                        InstanceState::Starting => "STARTING",
                        InstanceState::Running => "RUNNING",
                    },
                    Alignment::Left,
                );
        }
        view.print();
        Result::Ok(())
    }
}
