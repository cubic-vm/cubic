use crate::commands::{self, Command};
use crate::env::Environment;
use crate::error::Error;
use crate::fs::FS;
use crate::image::ImageStore;
use crate::instance::{InstanceStore, Target};
use crate::ssh_cmd::Russh;
use crate::view::Console;
use clap::Parser;

/// Connect to a virtual machine instance with SSH
#[derive(Parser)]
pub struct InstanceSshCommand {
    /// Target instance (format: [username@]instance, e.g. 'cubic@mymachine' or 'mymachine')
    pub target: Target,
    /// Execute a command in the virtual machine
    pub cmd: Option<String>,
}

impl Command for InstanceSshCommand {
    fn run(
        &self,
        console: &mut dyn Console,
        env: &Environment,
        image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<(), Error> {
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
        ssh.set_cmd(self.cmd.clone());
        ssh.shell(console, &user, ssh_port);
        Ok(())
    }
}
