#[cfg(test)]
pub mod tests {

    use crate::error::Error;
    use crate::instance::{Instance, InstanceState, InstanceStore};
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

        fn load(&self, name: &str) -> Result<Instance, Error> {
            self.instances
                .iter()
                .find(|i| i.name == name)
                .cloned()
                .ok_or(Error::UnknownInstance(name.to_string()))
        }

        fn store(&self, _instance: &Instance) -> Result<(), Error> {
            Result::Err(Error::UnknownCommand)
        }

        fn clone(&self, _instance: &Instance, _new_name: &str) -> Result<(), Error> {
            Result::Err(Error::UnknownCommand)
        }

        fn rename(&self, _instance: &mut Instance, _new_name: &str) -> Result<(), Error> {
            Result::Err(Error::UnknownCommand)
        }

        fn resize(&self, _instance: &mut Instance, _size: u64) -> Result<(), Error> {
            Result::Err(Error::UnknownCommand)
        }

        fn delete(&self, _instance: &Instance) -> Result<(), Error> {
            Result::Err(Error::UnknownCommand)
        }

        fn stop(&self, _instance: &Instance) -> Result<(), Error> {
            Result::Err(Error::UnknownCommand)
        }

        fn get_state(&self, _instance: &Instance) -> InstanceState {
            InstanceState::Stopped
        }

        fn is_running(&self, _instance: &Instance) -> bool {
            false
        }

        fn get_pid(&self, _instance: &Instance) -> Result<u64, ()> {
            Result::Err(())
        }

        fn get_monitor(&self, _instance: &Instance) -> Result<Monitor, Error> {
            Result::Err(Error::UnknownCommand)
        }
    }
}
