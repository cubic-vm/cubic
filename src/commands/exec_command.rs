use crate::commands::{self, Command};
use crate::error::Result;
use crate::fs::FS;
use crate::models::Target;
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
    #[clap(flatten)]
    pub env_args: commands::EnvArgs,
}

impl Command for ExecCommand {
    fn run(&self, console: &mut dyn Console, context: &commands::Context) -> Result<()> {
        let env = context.get_env();
        let name = self.target.get_instance();

        commands::StartCommand {
            qemu_args: None,
            wait: true,
            yes: commands::YesArg { value: false },
            instances: name.clone().into(),
        }
        .run(console, context)?;

        let instance = context.get_instance_store().load(name.as_str())?;
        let user = self
            .target
            .get_user()
            .map(|user| user.to_string())
            .unwrap_or_else(|| instance.user.to_string());
        let ssh_port = instance.ssh_port;
        let client_key = env.get_ssh_private_key_file(name.as_str());
        console.debug(&format!(
            "Executing on '{name}' as '{user}' on port {ssh_port} using key '{client_key}': {}",
            self.cmd
        ));
        let mut ssh = Russh::new();
        ssh.set_private_keys(env.get_home_ssh_private_key_paths(&FS::new()));
        ssh.set_cmd(Some(self.cmd.clone()));
        ssh.set_env_vars(self.env_args.env_vars.clone());
        ssh.shell(console, name.as_str(), &client_key, &user, ssh_port);
        Ok(())
    }
}
