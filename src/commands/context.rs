use crate::models::{Environment, Instance};
use crate::platform::System;
use crate::qemu::QemuMonitorClient;
use crate::view::Console;
use crate::vm::{VmClient, VmControl};
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

    // Prefers the supervisor API. Instances of older versions have no API
    // socket, and QEMU can outlive its supervisor, so a live monitor counts too.
    pub fn get_vm(&self, instance: &Instance) -> Option<Box<dyn VmControl>> {
        if let Ok(api) = VmClient::connect(self.system.as_ref(), &self.env, instance) {
            return Some(Box::new(api));
        }
        let monitor = self.get_monitor(instance);
        if !monitor.is_running() {
            return None;
        }
        Some(Box::new(monitor))
    }

    pub fn is_running(&self, instance: &Instance) -> bool {
        self.get_vm(instance).is_some()
    }

    /// A stopped --rm instance that its session never cleaned up
    pub fn is_stale(&self, instance: &Instance) -> bool {
        instance.auto_remove && !self.is_running(instance)
    }
}
