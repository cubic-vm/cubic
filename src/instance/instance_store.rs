use crate::error::Error;
use crate::instance::{Instance, InstanceState};
use crate::qemu::Monitor;
use std::str;

pub trait InstanceStore {
    fn get_instances(&self) -> Vec<String>;
    fn exists(&self, name: &str) -> bool;
    fn load(&self, name: &str) -> Result<Instance, Error>;
    fn store(&self, instance: &Instance) -> Result<(), Error>;

    fn clone(&self, instance: &Instance, new_name: &str) -> Result<(), Error>;
    fn rename(&self, instance: &mut Instance, new_name: &str) -> Result<(), Error>;
    fn resize(&self, instance: &mut Instance, size: u64) -> Result<(), Error>;
    fn delete(&self, instance: &Instance) -> Result<(), Error>;

    fn start(
        &self,
        instance: &Instance,
        qemu_args: &Option<String>,
        verbose: bool,
    ) -> Result<(), Error>;
    fn stop(&self, instance: &Instance) -> Result<(), Error>;
    fn get_state(&self, instance: &Instance) -> InstanceState;
    fn is_running(&self, instance: &Instance) -> bool;
    fn get_pid(&self, instance: &Instance) -> Result<u64, ()>;
    fn get_monitor(&self, instance: &Instance) -> Result<Monitor, Error>;
}
