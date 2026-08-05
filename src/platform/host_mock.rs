use crate::platform::{Host, SystemMock};
use std::collections::HashMap;

// The environment the host was started with, alongside what it reports about
// its own size.
pub struct HostMock {
    env_vars: HashMap<String, String>,
    total_memory: u64,
    available_memory: u64,
    cpu_count: u16,
}

impl Default for HostMock {
    // A roomy host, so a test that does not care about host size never hits a
    // resource limit by accident.
    fn default() -> Self {
        Self {
            env_vars: HashMap::new(),
            total_memory: 16 * 1024 * 1024 * 1024,
            available_memory: 16 * 1024 * 1024 * 1024,
            cpu_count: 8,
        }
    }
}

impl HostMock {
    fn add_env_var(&mut self, key: &str, value: &str) {
        self.env_vars.insert(key.to_string(), value.to_string());
    }

    fn get_env_var(&self, key: &str) -> Option<String> {
        self.env_vars.get(key).cloned()
    }

    fn set_resources(&mut self, total_memory: u64, available_memory: u64, cpu_count: u16) {
        self.total_memory = total_memory;
        self.available_memory = available_memory;
        self.cpu_count = cpu_count;
    }
}

impl SystemMock {
    pub fn add_env_var(mut self, key: &str, value: &str) -> Self {
        self.host.add_env_var(key, value);
        self
    }

    pub fn set_host_resources(
        mut self,
        total_memory: u64,
        available_memory: u64,
        cpu_count: u16,
    ) -> Self {
        self.host
            .set_resources(total_memory, available_memory, cpu_count);
        self
    }
}

impl Host for SystemMock {
    fn read_env_var(&self, key: &str) -> Option<String> {
        self.host.get_env_var(key)
    }

    fn get_total_memory(&self) -> u64 {
        self.host.total_memory
    }

    fn get_available_memory(&self) -> u64 {
        self.host.available_memory
    }

    fn get_cpu_count(&self) -> u16 {
        self.host.cpu_count
    }
}
