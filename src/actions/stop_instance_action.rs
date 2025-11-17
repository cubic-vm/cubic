use crate::error::Error;
use crate::instance::{Instance, InstanceStore};

pub struct StopInstanceAction {
    instance: Instance,
}

impl StopInstanceAction {
    pub fn new(instance: &Instance) -> Self {
        Self {
            instance: instance.clone(),
        }
    }

    pub fn run(&mut self, instance_dao: &dyn InstanceStore) -> Result<(), Error> {
        if !instance_dao.exists(&self.instance.name) {
            return Err(Error::UnknownInstance(self.instance.name.clone()));
        }

        if instance_dao.is_running(&self.instance) {
            instance_dao.get_monitor(&self.instance)?.shutdown()?;
        }

        Ok(())
    }

    pub fn is_done(&self, instance_dao: &dyn InstanceStore) -> bool {
        !instance_dao.is_running(&self.instance)
    }
}
