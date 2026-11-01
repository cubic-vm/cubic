use crate::commands::Context;
use crate::instance::InstanceSerializer;
use crate::models::{Environment, Instance};
use crate::platform::{Access, FileSystem, SystemMock};
use crate::view::Console;
use std::sync::Arc;

// Greeting plus replies for the handshake and a few commands
const QMP_REPLIES: &[u8] = b"{\"QMP\": {}}\n{\"return\": {}, \"id\": \"0\"}\n{\"return\": {}, \"id\": \"1\"}\n{\"return\": {}, \"id\": \"2\"}\n";

pub struct ContextBuilder {
    system: Arc<SystemMock>,
    env: Environment,
}

impl ContextBuilder {
    pub fn new(system: SystemMock) -> Self {
        Self {
            system: Arc::new(system),
            env: Environment::default(),
        }
    }

    pub fn get_system(&self) -> &Arc<SystemMock> {
        &self.system
    }

    pub fn get_env(&mut self) -> &mut Environment {
        &mut self.env
    }

    // Writes the config of each instance, so it loads like a created one
    pub fn add_instances(&self, instances: Vec<Instance>) {
        for instance in instances {
            let config = InstanceSerializer::new().serialize(&instance).unwrap();
            let dir = self.env.get_instance_dir2(&instance.name);
            let path = self.env.get_instance_toml_config_file(&instance.name);
            let content = config.as_bytes();
            self.system.create_dir(&dir).unwrap();
            self.system
                .write_file(&path, content, Access::Normal)
                .unwrap();
        }
    }

    // Opens a monitor socket for each name, so the instance counts as running
    pub fn set_running(&self, names: &[&str]) {
        for name in names {
            let socket = self.env.get_monitor_socket(name);
            self.system.insert_socket(socket, QMP_REPLIES);
        }
    }

    // Seeds the qemu-img info report for the image of `instance`
    pub fn add_image_info(&self, instance: &str, used: u64, capacity: u64, snapshots: &[&str]) {
        let snapshots: Vec<String> = snapshots
            .iter()
            .map(|name| format!(r#"{{"name": "{name}"}}"#))
            .collect();
        let info = format!(
            r#"{{"actual-size": {used}, "virtual-size": {capacity}, "snapshots": [{}]}}"#,
            snapshots.join(", ")
        );
        let image = self.env.get_instance_image_file(instance);
        let command = format!(
            "qemu-img info --force-share --output json {}",
            image.display()
        );
        self.system.insert_command_output(&command, info.as_bytes());
    }

    // Seeds a qemu-img snapshot call on the image of `instance` and returns it
    pub fn add_snapshot_command(&self, flag: &str, instance: &str, snapshot: &str) -> String {
        let image = self.env.get_instance_image_file(instance);
        let command = format!("qemu-img snapshot {flag} {snapshot} {}", image.display());
        self.system.insert_command_output(&command, b"");
        command
    }

    pub fn build(self) -> Context {
        Context::new(self.system.clone(), Console::new(self.system), self.env)
    }
}
