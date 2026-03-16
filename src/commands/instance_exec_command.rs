use crate::commands::{self, Command};
use crate::env::Environment;
use crate::error::Result;
use crate::fs::FS;
use crate::image::ImageStore;
use crate::instance::{InstanceStore, Target};
use crate::ssh_cmd::Russh;
use crate::view::Console;
use clap::Parser;

/// Execute a command in the virtual machine instance
#[derive(Parser)]
pub struct InstanceExecCommand {
    /// Target instance (format: [username@]instance, e.g. 'myinstance' or 'cubic@myinstance')
    pub target: Target,
    /// Command to execute in the virtual machine instance
    pub cmd: String,
}

impl Command for InstanceExecCommand {
    fn run(
        &self,
        console: &mut dyn Console,
        env: &Environment,
        image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<()> {
        let name = self.target.get_instance();

        commands::InstanceStartCommand {
            qemu_args: None,
            wait: true,
            instances: vec![name.to_string()],
        }
        .run(console, env, image_store, instance_store)?;

        let instance = instance_store.load(name.as_str())?;
        let user = self
            .target
            .get_user()
            .map(|user| user.to_string())
            .unwrap_or(instance.user.to_string());
        let ssh_port = instance.ssh_port;
        let mut ssh = Russh::new();
        ssh.set_private_keys(env.get_ssh_private_key_paths(&FS::new(), vec![name.to_string()]));
        ssh.set_cmd(Some(self.cmd.clone()));
        ssh.shell(console, &user, ssh_port);
        Ok(())
    }
}
