use crate::error::Result;
use crate::instance::Instance;
#[cfg(not(any(windows, test)))]
use crate::qemu::Monitor;
use std::str;

pub trait InstanceStore {
    fn get_instances(&self) -> Vec<String>;
    fn exists(&self, name: &str) -> bool;
    fn load(&self, name: &str) -> Result<Instance>;
    fn store(&self, instance: &Instance) -> Result<()>;

    fn rename(&self, instance: &mut Instance, new_name: &str) -> Result<()>;
    fn resize(&self, instance: &mut Instance, size: u64) -> Result<()>;
    fn delete(&self, instance: &Instance) -> Result<()>;

    fn is_running(&self, instance: &Instance) -> bool;
    fn get_pid(&self, instance: &Instance) -> std::result::Result<u64, ()>;

    #[cfg(not(any(windows, test)))]
    fn get_monitor(&self, instance: &Instance) -> Result<Monitor>;
}
