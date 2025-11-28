use crate::commands::{self, Command};
use crate::env::Environment;
use crate::error::Error;
use crate::image::ImageStore;
use crate::instance::{InstanceStore, Target};
use crate::ssh_cmd::{Openssh, Russh, Ssh, get_ssh_private_key_names};
use crate::view::Console;
use clap::Parser;
use std::env;

#[derive(Clone, Parser)]
pub struct SshArgs {
    /// Forward X over SSH
    #[clap(short = 'X', default_value_t = false)]
    pub xforward: bool,
    /// Select the ssh client library (openssh or russh are supported)
    #[clap(long, conflicts_with = "russh", default_value_t = false, hide = true)]
    pub openssh: bool,
    #[clap(long, conflicts_with = "openssh", default_value_t = false, hide = true)]
    pub russh: bool,
}

/// Connect to a virtual machine instance with SSH
#[derive(Parser)]
pub struct InstanceSshCommand {
    #[clap(flatten)]
    pub args: SshArgs,
    /// Pass additional SSH arguments
    #[clap(long)]
    pub ssh_args: Option<String>,
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

        let mut ssh: Box<dyn Ssh> = if !self.args.russh {
            Box::new(Openssh::new())
        } else {
            Box::new(Russh::new())
        };
        ssh.set_known_hosts_file(
            env::var("HOME")
                .map(|dir| format!("{dir}/.ssh/known_hosts"))
                .ok(),
        );
        ssh.set_private_keys(get_ssh_private_key_names()?);
        ssh.set_args(self.ssh_args.clone().unwrap_or_default());
        ssh.set_cmd(self.cmd.clone());
        ssh.shell(console, &user, ssh_port, self.args.xforward);
        Ok(())
    }
}
