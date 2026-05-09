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

const ABOUT: &str = "\
Cubic is a lightweight command line manager for virtual machines. It
has a simple, daemonless and rootless design. All Cubic virtual machines
run isolated in the user context. Cubic is built on top of QEMU, KVM and
cloud-init.

Examples:

  Create a new VM instance with:
  $ cubic create example --image ubuntu:noble
  Open a shell in the VM instance:
  $ cubic ssh example

  Alternatively, use `run` to execute the above commands in a single command:
  $ cubic run example --image ubuntu:noble

  Show all supported VM images:
  $ cubic images

  List previously created VM instances:
  $ cubic instances

  Show information about a VM instance:
  $ cubic show <instance>

  Execute a command in a VM instance:
  $ cubic exec <instance> <shell command>

  Transfer files and directories between host and VM instance:
  $ cubic scp <path/to/host/file> <instance>:<path/to/guest/file>
  See `cubic scp --help` for more examples

For more information, visit: https://cubic-vm.org/
The source code is located at: https://github.com/cubic-vm/cubic";

#[derive(Parser)]
#[command(
    author,
    version,
    about = ABOUT,
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
