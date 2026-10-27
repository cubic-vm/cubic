use crate::commands::Context;
use crate::instance::InstanceStoreMock;
use crate::models::Environment;
use crate::platform::SystemMock;
use crate::view::Console;
use std::sync::Arc;

pub struct ContextBuilder {
    system: Arc<SystemMock>,
    env: Environment,
    store: InstanceStoreMock,
}

impl ContextBuilder {
    pub fn new(system: SystemMock) -> Self {
        Self {
            system: Arc::new(system),
            env: Environment::default(),
            store: InstanceStoreMock::new(Vec::new()),
        }
    }

    pub fn get_system(&self) -> &Arc<SystemMock> {
        &self.system
    }

    pub fn get_env(&mut self) -> &mut Environment {
        &mut self.env
    }

    pub fn get_store(&mut self) -> &mut InstanceStoreMock {
        &mut self.store
    }

    pub fn build(self) -> Context {
        Context::new(
            self.system.clone(),
            Console::new(self.system),
            self.env,
            Box::new(self.store),
        )
    }
}
