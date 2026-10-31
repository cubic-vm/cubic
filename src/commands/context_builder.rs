use crate::commands::Context;
use crate::instance::InstanceStoreMock;
use crate::models::Environment;
use crate::platform::SystemMock;
use crate::view::Console;
use std::sync::Arc;

// Greeting plus replies for the handshake and a few commands
const QMP_REPLIES: &[u8] = b"{\"QMP\": {}}\n{\"return\": {}, \"id\": \"0\"}\n{\"return\": {}, \"id\": \"1\"}\n{\"return\": {}, \"id\": \"2\"}\n";

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

    // Opens a monitor socket for each name, so the instance counts as running
    pub fn set_running(&self, names: &[&str]) {
        for name in names {
            let socket = self.env.get_monitor_socket(name);
            self.system.insert_socket(socket, QMP_REPLIES);
        }
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
