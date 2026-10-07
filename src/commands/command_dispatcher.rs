use crate::commands::{self, Command};
use crate::env::EnvironmentFactory;
use crate::error::Result;
use crate::instance::InstanceDao;
use crate::platform::System;
use crate::view::{Console, Verbosity};
use clap::{CommandFactory, Parser, Subcommand};
use std::sync::Arc;

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
    Snapshot(commands::SnapshotCommand),
    Restore(commands::RestoreCommand),
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

const ABOUT: &str = "\
Cubic is a cross-platform tool that spins up Linux virtual machines with a
single command.

Create a VM instance and open a shell in it:
  $ cubic run -i ubuntu

Every command can be shortened while the short form stays unique:
  $ cubic in

For more information, visit: https://cubic-vm.org/
The source code is located at: https://github.com/cubic-vm/cubic";

#[derive(Parser)]
#[command(
    author,
    version,
    about = ABOUT,
    infer_subcommands = true,
    disable_help_subcommand = true
)]
pub struct CommandDispatcher {
    #[command(subcommand)]
    pub command: Option<commands::Commands>,

    #[clap(flatten)]
    global: GlobalOptions,
}

impl CommandDispatcher {
    pub async fn dispatch(self, system: Arc<dyn System>, console: &Arc<Console>) -> Result<u8> {
        let Some(command) = self.command else {
            console.print(&CommandDispatcher::command().render_long_help().to_string());
            return Ok(0);
        };

        console.set_verbosity(Verbosity::new(self.global.verbose, self.global.quiet));
        let env = EnvironmentFactory::create_env(system.as_ref())?;
        let context = &commands::Context::new(
            Arc::clone(&system),
            Arc::clone(console),
            env.clone(),
            Box::new(InstanceDao::new(Arc::clone(&system), &env)?),
        );

        let result = match &command {
            Commands::Run(cmd) => cmd.run(context).await,
            Commands::Instances(cmd) => cmd.run(context).await,
            Commands::Images(cmd) => cmd.run(context).await,
            Commands::Ports(cmd) => cmd.run(context).await,
            Commands::Create(cmd) => cmd.run(context).await,
            Commands::Modify(cmd) => cmd.run(context).await,
            Commands::Clone(cmd) => cmd.run(context).await,
            Commands::Snapshot(cmd) => cmd.run(context).await,
            Commands::Restore(cmd) => cmd.run(context).await,
            Commands::Rename(cmd) => cmd.run(context).await,
            Commands::Show(cmd) => cmd.run(context).await,
            Commands::Start(cmd) => cmd.run(context).await,
            Commands::Stop(cmd) => cmd.run(context).await,
            Commands::Restart(cmd) => cmd.run(context).await,
            Commands::Console(cmd) => cmd.run(context).await,
            Commands::Ssh(cmd) => cmd.run(context).await,
            Commands::Scp(cmd) => cmd.run(context).await,
            Commands::Exec(cmd) => cmd.run(context).await,
            Commands::Delete(cmd) => cmd.run(context).await,
            Commands::Prune(cmd) => cmd.run(context).await,
            Commands::Completions(cmd) => cmd.run(context).await,
        };

        // Clear any animation the command left running, including on error.
        console.clear_animation();
        result
    }
}
