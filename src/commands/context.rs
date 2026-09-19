use crate::instance::InstanceStore;
use crate::models::Environment;
use crate::platform::System;
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
}
