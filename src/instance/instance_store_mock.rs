#[cfg(test)]
pub mod tests {

    use crate::error::{Error, Result};
    use crate::instance::InstanceStore;
    use crate::models::Instance;
    use crate::qemu::Monitor;

    pub struct InstanceStoreMock {
        instances: Vec<Instance>,
    }

    impl InstanceStoreMock {
        pub fn new(instances: Vec<Instance>) -> Self {
            Self { instances }
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

        fn is_running(&self, _instance: &Instance) -> bool {
            false
        }

        fn get_pid(&self, _instance: &Instance) -> std::result::Result<u64, ()> {
            std::result::Result::Err(())
        }

        fn kill(&self, _instance: &Instance) -> Result<()> {
            Ok(())
        }

        fn get_monitor(&self, instance: &Instance) -> Result<Monitor> {
            Err(Error::InstanceNotRunning(instance.name.clone()))
        }
    }
}
