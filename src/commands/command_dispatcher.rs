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
Cubic runs Linux virtual machines on Linux, macOS and Windows with a single
command.

Every distribution comes as an official image and is ready to use within
seconds, so you skip the long installation. Cubic keeps things simple and secure
by acting as lightweight glue over proven tools. No privileged system service is
required and every VM runs as your normal user. Cubic is built on top of QEMU,
EDK2, official Linux distribution images and cloud-init.

Examples:

  Create a new VM instance with:
  $ cubic create example --image ubuntu
  Open a shell in the VM instance:
  $ cubic ssh example

  Alternatively, use `run` to execute the above commands in a single command:
  $ cubic run example --image ubuntu

  Show all supported VM images:
  $ cubic images

  List previously created VM instances:
  $ cubic instances

  Show information about a VM instance:
  $ cubic show <instance>

  Execute a command in a VM instance:
  $ cubic exec <instance> -- <command>

  Transfer files and directories between host and VM instance:
  $ cubic scp <path/to/host/file> <instance>:<path/to/guest/file>
  See `cubic scp --help` for more examples

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
