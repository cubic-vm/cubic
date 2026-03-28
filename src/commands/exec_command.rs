use crate::commands::{self, Command, Iso9660};
use crate::env::Environment;
use crate::error::Result;
use crate::fs::FS;
use crate::image::ImageStore;
use crate::instance::{InstanceStore, Target};
use crate::ssh_cmd::Russh;
use crate::view::Console;
use clap::Parser;

/// Execute commands on VM instances
///
/// Examples:
///
///   Update a VM instance:
///   $ cubic exec noble "sudo apt update && sudo apt full-upgrade -y"
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct ExecCommand {
    /// Target instance (format: [username@]instance, e.g. 'myinstance' or 'cubic@myinstance')
    pub target: Target,
    /// Command to execute in the virtual machine instance
    pub cmd: String,
    /// Switch for Rust and system ISO9600 implementation
    #[clap(hide = true)]
    #[arg(value_enum, default_value_t = Iso9660::System)]
    pub iso9660: Iso9660,
}

impl Command for ExecCommand {
    fn run(
        &self,
        console: &mut dyn Console,
        env: &Environment,
        image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<()> {
        let name = self.target.get_instance();

        commands::StartCommand {
            qemu_args: None,
            wait: true,
            instances: vec![name.to_string()],
            iso9660: self.iso9660.clone(),
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
