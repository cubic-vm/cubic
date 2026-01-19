use crate::commands::{
    self, Command, InstanceCloneCommand, InstanceModifyCommand, InstanceRenameCommand,
};
use crate::env::EnvironmentFactory;
use crate::error::Error;
use crate::image::ImageDao;
use crate::instance::InstanceDao;
use crate::view::Console;
use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    Run(commands::InstanceRunCommand),
    Create(commands::CreateInstanceCommand),
    Instances(commands::ListInstanceCommand),
    Images(commands::ListImageCommand),
    Ports(commands::ListPortCommand),
    Show(commands::ShowCommand),
    Modify(InstanceModifyCommand),
    Console(commands::InstanceConsoleCommand),
    Ssh(commands::InstanceSshCommand),
    Scp(commands::InstanceScpCommand),
    Start(commands::InstanceStartCommand),
    Stop(commands::InstanceStopCommand),
    Restart(commands::InstanceRestartCommand),
    Rename(InstanceRenameCommand),
    Clone(InstanceCloneCommand),
    Delete(commands::DeleteInstanceCommand),
    Prune(commands::PruneCommand),
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
#[command(author, version, about, long_about = None, arg_required_else_help = true, infer_subcommands = true)]
pub struct CommandDispatcher {
    #[command(subcommand)]
    pub command: Commands,

    #[clap(flatten)]
    global: GlobalOptions,
}

impl CommandDispatcher {
    pub fn dispatch(self, console: &mut dyn Console) -> Result<(), Error> {
        console.set_verbosity(commands::Verbosity::new(
            self.global.verbose,
            self.global.quiet,
        ));
        let env = EnvironmentFactory::create_env()?;
        let image_dao = ImageDao::new(&env)?;
        let instance_dao = InstanceDao::new(&env)?;

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
            Commands::Delete(cmd) => cmd,
            Commands::Prune(cmd) => cmd,
        }
        .run(console, &env, &image_dao, &instance_dao)
    }
}
