#[cfg(test)]
pub mod tests {

    use crate::error::{Error, Result};
    use crate::instance::InstanceStore;
    use crate::models::Instance;
    use crate::qemu::Monitor;
    use std::sync::Mutex;

    pub struct InstanceStoreMock {
        instances: Vec<Instance>,
        running: Vec<String>,
        killed: Mutex<Vec<String>>,
    }

    impl InstanceStoreMock {
        pub fn new(instances: Vec<Instance>) -> Self {
            Self::new_with_running(instances, &[])
        }

        pub fn new_with_running(instances: Vec<Instance>, running: &[&str]) -> Self {
            Self {
                instances,
                running: running.iter().map(|name| name.to_string()).collect(),
                killed: Mutex::new(Vec::new()),
            }
        }

        pub fn get_killed(&self) -> Vec<String> {
            self.killed.lock().unwrap().clone()
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

        fn store(&self, _instance: &Instance) -> Result<()> {
            Ok(())
        }

        fn rename(&self, _instance: &mut Instance, _new_name: &str) -> Result<()> {
            Ok(())
        }

        fn resize(&self, _instance: &mut Instance, _size: u64) -> Result<()> {
            Ok(())
        }

        fn delete(&self, _instance: &Instance) -> Result<()> {
            Ok(())
        }

        fn is_running(&self, instance: &Instance) -> bool {
            self.running.contains(&instance.name)
        }

        fn get_pid(&self, _instance: &Instance) -> Option<u64> {
            None
        }

        fn kill(&self, instance: &Instance) -> Result<()> {
            self.killed.lock().unwrap().push(instance.name.clone());
            Ok(())
        }

        fn get_monitor(&self, instance: &Instance) -> Result<Monitor> {
            Err(Error::InstanceNotRunning(instance.name.clone()))
        }
    }
}
