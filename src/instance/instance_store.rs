use crate::error::{Error, Result};
use crate::models::Instance;
use crate::qemu::QemuMonitorClient;
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

    fn is_running(&self, instance: &Instance) -> bool;

    /// A stopped --rm instance that its session never cleaned up
    fn is_stale(&self, instance: &Instance) -> bool {
        instance.auto_remove && !self.is_running(instance)
    }

    /// Frees a name that only a stale instance still holds
    fn claim_name(&self, name: &str) -> Result<()> {
        match self.load(name) {
            Err(Error::UnknownInstance(_)) => Ok(()),
            Ok(instance) if self.is_stale(&instance) => self.delete(&instance),
            _ => Err(Error::InstanceAlreadyExists(name.to_string())),
        }
    }

    fn get_pid(&self, instance: &Instance) -> Option<u64>;
    fn kill(&self, instance: &Instance) -> Result<()>;

    fn get_monitor(&self, instance: &Instance) -> Result<QemuMonitorClient>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::InstanceStoreMock;
    use std::sync::Arc;

    #[test]
    fn test_claim_name_deletes_a_stale_instance() {
        let store = InstanceStoreMock::new(vec![Instance {
            name: "test".to_string(),
            auto_remove: true,
            ..Instance::default()
        }]);
        let deleted = Arc::clone(&store.deleted);

        store.claim_name("test").unwrap();

        assert_eq!(*deleted.lock().unwrap(), ["test"]);
    }

    #[test]
    fn test_claim_name_rejects_a_running_auto_remove_instance() {
        let store = InstanceStoreMock::new_with_running(
            vec![Instance {
                name: "test".to_string(),
                auto_remove: true,
                ..Instance::default()
            }],
            &["test"],
        );

        let result = store.claim_name("test");

        assert!(matches!(
            result,
            Err(Error::InstanceAlreadyExists(ref name)) if name == "test"
        ));
    }
}
