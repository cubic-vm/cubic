#[cfg(test)]
pub mod tests {

    use crate::error::{Error, Result};
    use crate::instance::InstanceStore;
    use crate::models::Instance;
    use crate::qemu::QemuMonitorClient;
    use std::sync::{Arc, Mutex};

    pub struct InstanceStoreMock {
        instances: Vec<Instance>,
        running: Vec<String>,
        // Shared, so a test keeps a handle on what the store recorded after it
        // moved into a Context.
        pub killed: Arc<Mutex<Vec<String>>>,
        pub stored: Arc<Mutex<Vec<Instance>>>,
        pub deleted: Arc<Mutex<Vec<String>>>,
        // Records the snapshot calls as "<action> <instance>/<snapshot>".
        pub snapshots: Arc<Mutex<Vec<String>>>,
    }

    impl InstanceStoreMock {
        pub fn new(instances: Vec<Instance>) -> Self {
            Self::new_with_running(instances, &[])
        }

        pub fn new_with_running(instances: Vec<Instance>, running: &[&str]) -> Self {
            Self {
                instances,
                running: running.iter().map(|name| name.to_string()).collect(),
                killed: Arc::new(Mutex::new(Vec::new())),
                stored: Arc::new(Mutex::new(Vec::new())),
                deleted: Arc::new(Mutex::new(Vec::new())),
                snapshots: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn record_snapshot(&self, action: &str, instance: &Instance, name: &str) {
            self.snapshots
                .lock()
                .unwrap()
                .push(format!("{action} {}/{name}", instance.name));
        }
    }

    impl InstanceStore for InstanceStoreMock {
        fn get_instances(&self) -> Vec<String> {
            self.instances.iter().map(|i| i.name.clone()).collect()
        }

        fn exists(&self, name: &str) -> bool {
            self.instances.iter().any(|i| i.name == name)
        }

        fn load(&self, name: &str) -> Result<Instance> {
            self.instances
                .iter()
                .find(|i| i.name == name)
                .cloned()
                .ok_or(Error::UnknownInstance(name.to_string()))
        }

        fn store(&self, instance: &Instance) -> Result<()> {
            self.stored.lock().unwrap().push(instance.clone());
            Ok(())
        }

        fn rename(&self, _instance: &mut Instance, _new_name: &str) -> Result<()> {
            Ok(())
        }

        fn resize(&self, _instance: &mut Instance, _size: u64) -> Result<()> {
            Ok(())
        }

        fn delete(&self, instance: &Instance) -> Result<()> {
            self.deleted.lock().unwrap().push(instance.name.clone());
            Ok(())
        }

        fn create_snapshot(&self, instance: &Instance, name: &str) -> Result<()> {
            self.record_snapshot("create", instance, name);
            Ok(())
        }

        fn restore_snapshot(&self, instance: &Instance, name: &str) -> Result<()> {
            self.record_snapshot("restore", instance, name);
            Ok(())
        }

        fn delete_snapshot(&self, instance: &Instance, name: &str) -> Result<()> {
            self.record_snapshot("delete", instance, name);
            Ok(())
        }

        fn is_running(&self, instance: &Instance) -> bool {
            self.running.contains(&instance.name)
                && !self.killed.lock().unwrap().contains(&instance.name)
        }

        fn kill(&self, instance: &Instance) -> Result<()> {
            self.killed.lock().unwrap().push(instance.name.clone());
            Ok(())
        }

        fn get_monitor(&self, instance: &Instance) -> Result<QemuMonitorClient> {
            Err(Error::InstanceNotRunning(instance.name.clone()))
        }
    }
}
