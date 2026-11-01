use crate::actions::{DeleteInstanceAction, LoadInstanceAction};
use crate::error::{Error, Result};
use crate::models::{Environment, Instance};
use crate::platform::System;
use crate::qemu::QemuMonitorClient;
use crate::view::Console;
use std::sync::Arc;

pub struct Context {
    system: Arc<dyn System>,
    console: Arc<Console>,
    env: Environment,
}

impl Context {
    pub fn new(system: Arc<dyn System>, console: Arc<Console>, env: Environment) -> Self {
        Self {
            system,
            console,
            env,
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

    pub fn has_instance(&self, name: &str) -> bool {
        self.system.exists_path(&self.env.get_instance_dir2(name))
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
        match LoadInstanceAction::new().read(self, name) {
            Err(Error::UnknownInstance(_)) => Ok(()),
            Ok(instance) if self.is_stale(&instance) => {
                DeleteInstanceAction::new().run(self, &instance)
            }
            _ => Err(Error::InstanceAlreadyExists(name.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::ContextBuilder;
    use crate::error::Error;
    use crate::models::{Environment, Instance};
    use crate::platform::{FileSystem, SystemMock};
    use std::sync::Arc;

    fn build_builder() -> ContextBuilder {
        let builder = ContextBuilder::new(SystemMock::new());
        builder.add_instances(vec![Instance {
            name: "test".to_string(),
            auto_remove: true,
            ..Instance::default()
        }]);
        builder
    }

    #[test]
    fn test_claim_name_deletes_a_stale_instance() {
        let builder = build_builder();
        let system = Arc::clone(builder.get_system());

        builder.build().claim_name("test").unwrap();

        let dir = Environment::default().get_instance_dir2("test");
        assert!(!system.exists_path(&dir));
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
