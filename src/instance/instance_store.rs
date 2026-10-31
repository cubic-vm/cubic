use crate::error::Result;
use crate::models::Instance;
use std::str;

pub trait InstanceStore {
    fn get_instances(&self) -> Vec<String>;
    fn exists(&self, name: &str) -> bool;
    fn load(&self, name: &str) -> Result<Instance>;
    fn store(&self, instance: &Instance) -> Result<()>;

    fn rename(&self, instance: &mut Instance, new_name: &str) -> Result<()>;
    fn resize(&self, instance: &mut Instance, size: u64) -> Result<()>;
    fn delete(&self, instance: &Instance) -> Result<()>;

    fn create_snapshot(&self, instance: &Instance, name: &str) -> Result<()>;
    fn restore_snapshot(&self, instance: &Instance, name: &str) -> Result<()>;
    fn delete_snapshot(&self, instance: &Instance, name: &str) -> Result<()>;
}
