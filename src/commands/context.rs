use crate::error::{Error, Result};
use crate::instance::InstanceStore;
use crate::models::{Environment, Instance};
use crate::platform::System;
use crate::qemu::QemuMonitorClient;
use crate::view::Console;
use std::sync::Arc;

pub struct Context {
    system: Arc<dyn System>,
    console: Arc<Console>,
    env: Environment,
    instance_store: Box<dyn InstanceStore>,
}

impl Context {
    pub fn new(
        system: Arc<dyn System>,
        console: Arc<Console>,
        env: Environment,
        instance_store: Box<dyn InstanceStore>,
    ) -> Self {
        Self {
            system,
            console,
            env,
            instance_store,
        }
    }

    pub fn get_system(&self) -> &dyn System {
        self.system.as_ref()
    }

    pub fn get_console(&self) -> &Arc<Console> {
        &self.console
    }

    pub fn get_env(&self) -> &Environment {
        &self.env
    }

    pub fn get_instance_store(&self) -> &dyn InstanceStore {
        self.instance_store.as_ref()
    }

    pub fn get_monitor(&self, instance: &Instance) -> QemuMonitorClient {
        QemuMonitorClient::new(self.system.clone(), &self.env, instance)
    }

    /// A stopped --rm instance that its session never cleaned up
    pub fn is_stale(&self, instance: &Instance) -> bool {
        instance.auto_remove && !self.get_monitor(instance).is_running()
    }

    /// Frees a name that only a stale instance still holds
    pub fn claim_name(&self, name: &str) -> Result<()> {
        let store = self.get_instance_store();
        match store.load(name) {
            Err(Error::UnknownInstance(_)) => Ok(()),
            Ok(instance) if self.is_stale(&instance) => store.delete(&instance),
            _ => Err(Error::InstanceAlreadyExists(name.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::ContextBuilder;
    use crate::error::Error;
    use crate::models::Instance;
    use crate::platform::SystemMock;
    use std::sync::Arc;

    fn build_builder() -> ContextBuilder {
        let mut builder = ContextBuilder::new(SystemMock::new());
        builder.get_store().add_instances(vec![Instance {
            name: "test".to_string(),
            auto_remove: true,
            ..Instance::default()
        }]);
        builder
    }

    #[test]
    fn test_claim_name_deletes_a_stale_instance() {
        let mut builder = build_builder();
        let deleted = Arc::clone(&builder.get_store().deleted);

        builder.build().claim_name("test").unwrap();

        assert_eq!(*deleted.lock().unwrap(), ["test"]);
    }

    #[test]
    fn test_claim_name_rejects_a_running_auto_remove_instance() {
        let builder = build_builder();
        builder.set_running(&["test"]);

        let result = builder.build().claim_name("test");

        assert!(matches!(
            result,
            Err(Error::InstanceAlreadyExists(ref name)) if name == "test"
        ));
    }
}
