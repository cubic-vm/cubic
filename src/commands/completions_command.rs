use crate::commands::{Command, CommandDispatcher};
use crate::env::Environment;
use crate::error::Error;
use crate::image::ImageStore;
use crate::instance::InstanceStore;
use crate::view::Console;
use clap::{CommandFactory, Parser};
use clap_complete::Shell;

/// Generate command completions for your shell
#[derive(Parser)]
#[clap(alias = "completion")]
pub struct CompletionsCommand {
    /// The shell to generate completions for
    shell: Option<Shell>,
}

impl Command for CompletionsCommand {
    fn run(
        &self,
        _console: &mut dyn Console,
        _env: &Environment,
        _image_store: &dyn ImageStore,
        _instance_store: &dyn InstanceStore,
    ) -> Result<(), Error> {
        let Some(shell) = self.shell.or_else(Shell::from_env) else {
            return Err(Error::CouldNotDetectShell);
        };

        let mut cmd = CommandDispatcher::command();
        let name = cmd.get_name().to_string();
        clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
        Ok(())
    }
}
