use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore};
use crate::util;

pub struct InstanceConfigCommand {
    instance: String,
    cpus: Option<u16>,
    mem: Option<String>,
    disk: Option<String>,
}

impl InstanceConfigCommand {
    pub fn new(
        instance: &str,
        cpus: &Option<u16>,
        mem: &Option<String>,
        disk: &Option<String>,
    ) -> Self {
        Self {
            instance: instance.to_string(),
            cpus: cpus.as_ref().cloned(),
            mem: mem.as_ref().cloned(),
            disk: disk.as_ref().cloned(),
        }
    }

    pub fn run(&self, instance_dao: &InstanceDao) -> Result<(), Error> {
        let mut instance = instance_dao.load(&self.instance)?;

        if let Some(cpus) = &self.cpus {
            instance.cpus = *cpus;
        }

        if let Some(mem) = &self.mem {
            instance.mem = util::human_readable_to_bytes(mem)?;
        }

        if let Some(disk) = &self.disk {
            instance_dao.resize(&mut instance, util::human_readable_to_bytes(disk)?)?;
        }

        instance_dao.store(&instance)?;
        Result::Ok(())
    }
}
