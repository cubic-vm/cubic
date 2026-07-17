use crate::commands::{self, Command};
use crate::error::Result;
use crate::fs::FS;
use crate::models::Target;
use crate::ssh_cmd::Russh;
use crate::view::Console;
use clap::Parser;

/// Connect to VM instances
///
/// Examples:
///
///   Connect to VM instance 'my-instance':
///   $ cubic ssh my-instance
///   [...]
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct SshCommand {
    /// Target instance (format: [username@]instance, e.g. 'myinstance' or 'cubic@myinstance')
    pub target: Target,
    #[clap(flatten)]
    pub env_args: commands::EnvArgs,
}

impl Command for SshCommand {
    fn run(&self, console: &mut dyn Console, context: &commands::Context) -> Result<()> {
        let env = context.get_env();
        let instance_store = context.get_instance_store();

        let name = self.target.get_instance();

        commands::StartCommand {
            qemu_args: None,
            wait: true,
            yes: commands::YesArg { value: false },
            instances: name.clone().into(),
        }
        .run(console, context)?;

        let instance = instance_store.load(name.as_str())?;
        let user = self
            .target
            .get_user()
            .map(|user| user.to_string())
            .unwrap_or_else(|| instance.user.to_string());
        let ssh_port = instance.ssh_port;
        let client_key = env.get_ssh_private_key_file(name.as_str());
        console.debug(&format!(
            "Connecting to '{name}' as '{user}' on port {ssh_port} using key '{client_key}'"
        ));
        let mut ssh = Russh::new(context);
        ssh.set_private_keys(env.get_home_ssh_private_key_paths(context.get_system(), &FS::new()));
        ssh.set_env_vars(self.env_args.env_vars.clone());
        ssh.shell(console, name.as_str(), &client_key, &user, ssh_port);
        Ok(())
    }
}
