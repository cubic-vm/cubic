use crate::commands::{Command, CommandDispatcher, Context};
use crate::error::{Error, Result};
use crate::view::Console;
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
    fn run(&self, _console: &mut dyn Console, _context: &Context) -> Result<()> {
        let Some(shell) = self.shell.or_else(Shell::from_env) else {
            return Err(Error::CouldNotDetectShell);
        };

        let mut cmd = CommandDispatcher::command();
        let name = cmd.get_name().to_string();
        clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
        Ok(())
    }
}
