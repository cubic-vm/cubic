use crate::actions::LoadInstanceAction;
use crate::commands::{self, Command};
use crate::error::Result;
use crate::models::Target;
use crate::ssh::SshClient;
use crate::view::Spinner;
use clap::Parser;
use std::sync::Arc;

/// Connect to a VM instance
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
    async fn run(&self, context: &commands::Context) -> Result<u8> {
        let console = context.get_console();
        let env = context.get_env();

        let name = self.target.get_instance();

        commands::StartCommand {
            qemu_args: None,
            accel: self.accel,
            wait: true,
            yes: commands::YesArg { value: false },
            instances: name.clone().into(),
        }
        .run(context)
        .await?;

        let instance = LoadInstanceAction::new().run(context, name.as_str())?;
        let text = format!("Connecting to {}", instance.name);
        let mut spinner = Spinner::new(Arc::clone(console), text);

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
        let channel = ssh
            .open_channel(&instance.name, &client_key, &user, ssh_port)
            .await?;
        spinner.stop();
        ssh.shell(&instance.name, channel).await
    }
}
