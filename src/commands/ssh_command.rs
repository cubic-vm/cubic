use crate::actions::LoadInstanceAction;
use crate::commands::{self, Command};
use crate::error::Result;
use crate::models::Target;
use crate::ssh::SshClient;
use crate::util;
use crate::view::{Console, Spinner};
use clap::Parser;
use std::sync::{Arc, Mutex};

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
    pub accel: commands::AccelArg,
    #[clap(flatten)]
    pub env_args: commands::EnvArgs,
}

impl Command for SshCommand {
    fn run(&self, console: &mut Console<'_>, context: &commands::Context) -> Result<()> {
        let env = context.get_env();

        let name = self.target.get_instance();

        commands::StartCommand {
            qemu_args: None,
            accel: self.accel,
            wait: true,
            yes: commands::YesArg { value: false },
            instances: name.clone().into(),
        }
        .run(console, context)?;

        let instance = LoadInstanceAction::new().run(context, console, name.as_str())?;
        console.play(Arc::new(Mutex::new(Spinner::new(format!(
            "Connecting to {}",
            instance.name
        )))));

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
        let mut ssh = SshClient::new(context);
        ssh.set_private_keys(env.get_home_ssh_private_key_paths(context.get_system()));
        ssh.set_env_vars(self.env_args.env_vars.clone());
        let async_caller = util::AsyncCaller::new();
        let channel = async_caller.call(ssh.open_channel(
            console,
            &instance.name,
            &client_key,
            &user,
            ssh_port,
        ))?;
        console.stop();
        async_caller.call(ssh.shell(console, &instance.name, channel))?;
        Ok(())
    }
}
