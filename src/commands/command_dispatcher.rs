use crate::commands::{self, Command};
use crate::env::EnvironmentFactory;
use crate::error::Result;
use crate::image::ImageDao;
use crate::instance::InstanceDao;
use crate::view::Console;
use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    Run(commands::RunCommand),
    Create(commands::CreateCommand),
    Instances(commands::ListInstanceCommand),
    Images(commands::ListImageCommand),
    Ports(commands::ListPortCommand),
    Show(commands::ShowCommand),
    Modify(commands::ModifyCommand),
    Console(commands::ConsoleCommand),
    Ssh(commands::SshCommand),
    Scp(commands::ScpCommand),
    Exec(commands::ExecCommand),
    Start(commands::StartCommand),
    Stop(commands::StopCommand),
    Restart(commands::RestartCommand),
    Rename(commands::RenameCommand),
    Clone(commands::CloneCommand),
    Delete(commands::DeleteCommand),
    Prune(commands::PruneCommand),
    Completions(commands::CompletionsCommand),
}

#[derive(Parser, Default)]
pub struct GlobalOptions {
    /// Increase logging output
    #[clap(short, long, action, global = true)]
    verbose: bool,
    /// Reduce logging output
    #[clap(short, long, action, global = true)]
    quiet: bool,
}

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    arg_required_else_help = true,
    infer_subcommands = true,
    disable_help_subcommand = true
)]
pub struct CommandDispatcher {
    #[command(subcommand)]
    pub command: commands::Commands,

    #[clap(flatten)]
    global: GlobalOptions,
}

impl CommandDispatcher {
    pub fn dispatch(self, console: &mut dyn Console) -> Result<()> {
        console.set_verbosity(commands::Verbosity::new(
            self.global.verbose,
            self.global.quiet,
        ));
        let env = EnvironmentFactory::create_env()?;
        let context = &commands::Context::new(
            env.clone(),
            Box::new(ImageDao::new(&env)?),
            Box::new(InstanceDao::new(&env)?),
        );

        match &self.command {
            Commands::Run(cmd) => cmd as &dyn Command,
            Commands::Instances(cmd) => cmd,
            Commands::Images(cmd) => cmd,
            Commands::Ports(cmd) => cmd,
            Commands::Create(cmd) => cmd,
            Commands::Modify(cmd) => cmd,
            Commands::Clone(cmd) => cmd,
            Commands::Rename(cmd) => cmd,
            Commands::Show(cmd) => cmd,
            Commands::Start(cmd) => cmd,
            Commands::Stop(cmd) => cmd,
            Commands::Restart(cmd) => cmd,
            Commands::Console(cmd) => cmd,
            Commands::Ssh(cmd) => cmd,
            Commands::Scp(cmd) => cmd,
            Commands::Exec(cmd) => cmd,
            Commands::Delete(cmd) => cmd,
            Commands::Prune(cmd) => cmd,
            Commands::Completions(cmd) => cmd,
        }
        .run(console, context)
    }
}
