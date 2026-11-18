use crate::error::Result;
use crate::models::{Environment, Instance};
use crate::platform::System;
use crate::qemu::QemuMonitorClient;
use crate::view::Console;
use crate::vm::VmClient;
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

    pub fn get_vm(&self, instance: &Instance) -> Result<VmClient> {
        VmClient::connect(self.system.as_ref(), &self.env, instance)
    }

    // QEMU can outlive its supervisor, so a live monitor counts as running too.
    pub fn is_running(&self, instance: &Instance) -> bool {
        self.get_vm(instance).is_ok() || self.get_monitor(instance).is_running()
    }

    /// A stopped --rm instance that its session never cleaned up
    pub fn is_stale(&self, instance: &Instance) -> bool {
        instance.auto_remove && !self.is_running(instance)
    }
}
