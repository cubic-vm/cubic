use crate::error::Error;
use crate::instance::Instance;
#[cfg(not(windows))]
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

    fn is_running(&self, instance: &Instance) -> bool;
    fn get_pid(&self, instance: &Instance) -> Result<u64, ()>;

    #[cfg(not(windows))]
    fn get_monitor(&self, instance: &Instance) -> Result<Monitor, Error>;
}
