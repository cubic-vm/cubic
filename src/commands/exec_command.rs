use crate::actions::LoadInstanceAction;
use crate::commands::{self, Command};
use crate::error::Result;
use crate::models::Target;
use crate::ssh::SshClient;
use clap::Parser;

/// Execute a command in a VM instance
///
/// The words of the command are joined with a space and run by the shell of the
/// VM instance, so all three forms below send the same command. Options of
/// Cubic itself have to come before the command.
///
/// Examples:
///
///   Run a command in a VM instance:
///   $ cubic exec noble -- echo -ne hello
///   $ cubic exec noble echo -ne hello
///   $ cubic exec noble "echo -ne hello"
///
///   Quote the command to use the operators of the shell:
///   $ cubic exec noble "sudo apt update && sudo apt full-upgrade -y"
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct ExecCommand {
    /// Target instance (format: [username@]instance, e.g. 'myinstance' or 'cubic@myinstance')
    pub target: Target,
    /// Command to execute in the virtual machine instance
    #[clap(trailing_var_arg = true, allow_hyphen_values = true, num_args = 1.., required = true)]
    pub cmd: Vec<String>,
    #[clap(flatten)]
    pub accel: commands::AccelArg,
    #[clap(flatten)]
    pub env_args: commands::EnvArgs,
}

impl ExecCommand {
    // The guest shell parses the result, so a plain join is enough.
    fn build_cmd(&self) -> String {
        self.cmd.join(" ")
    }
}

impl Command for ExecCommand {
    async fn run(&self, context: &commands::Context) -> Result<u8> {
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
        let user = self
            .target
            .get_user()
            .map(|user| user.to_string())
            .unwrap_or_else(|| instance.user.to_string());
        let ssh_port = instance.ssh_port;
        let client_key = env.get_ssh_private_key_file(name.as_str());
        let cmd = self.build_cmd();
        context.get_console().debug(&format!(
            "Executing on '{name}' as '{user}' on port {ssh_port} using key '{client_key}': {cmd}"
        ));
        let mut ssh = SshClient::new(context);
        ssh.set_private_keys(env.get_home_ssh_private_key_paths(context.get_system()));
        ssh.set_cmd(Some(cmd));
        ssh.set_env_vars(self.env_args.env_vars.clone());
        let channel = ssh
            .open_channel(&instance.name, &client_key, &user, ssh_port)
            .await?;
        ssh.shell(name.as_str(), channel).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_cmd_from_unquoted_words() {
        let command =
            ExecCommand::try_parse_from(["exec", "vm", "--", "echo", "-ne", "hello"]).unwrap();
        assert_eq!(command.build_cmd(), "echo -ne hello");
    }

    #[test]
    fn test_build_cmd_keeps_a_quoted_command_unchanged() {
        let command =
            ExecCommand::try_parse_from(["exec", "vm", "sudo apt update && sudo apt upgrade"])
                .unwrap();
        assert_eq!(command.build_cmd(), "sudo apt update && sudo apt upgrade");
    }

    #[test]
    fn test_options_are_parsed_before_the_command() {
        let command =
            ExecCommand::try_parse_from(["exec", "vm", "--env", "FOO", "echo", "hi"]).unwrap();
        assert_eq!(command.env_args.env_vars, ["FOO"]);
        assert_eq!(command.build_cmd(), "echo hi");
    }
}
