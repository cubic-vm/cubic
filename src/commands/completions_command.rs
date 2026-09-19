use crate::commands::{Command, CommandDispatcher, Context};
use crate::error::{Error, Result};
use clap::{CommandFactory, Parser};
use clap_complete::Shell;

/// Generate shell completion scripts
///
/// Examples:
///
///   Generate Bash auto completion script:
///   $ cubic completions bash > /etc/bash_completion.d/cubic.bash
///
///   Generate Fish auto completion script:
///   $ cubic completions fish > ~/.config/fish/completions/cubic.fish
///
#[derive(Parser)]
#[clap(alias = "completion", verbatim_doc_comment)]
pub struct CompletionsCommand {
    /// The shell to generate completions for
    shell: Option<Shell>,
}

impl Command for CompletionsCommand {
    async fn run(&self, context: &Context) -> Result<u8> {
        let Some(shell) = self.shell.or_else(Shell::from_env) else {
            return Err(Error::CouldNotDetectShell);
        };

        let mut cmd = CommandDispatcher::command();
        let name = cmd.get_name().to_string();
        let mut script = Vec::new();
        clap_complete::generate(shell, &mut cmd, name, &mut script);
        context
            .get_console()
            .print(String::from_utf8_lossy(&script).trim_end());
        Ok(0)
    }
}
