use crate::commands::{
    self, InstanceAddCommand, InstanceCloneCommand, InstanceListCommand, InstanceModifyCommand,
    InstanceRenameCommand,
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
    Add(InstanceAddCommand),
    #[clap(alias = "list")]
    Ls(InstanceListCommand),
    Images(commands::ListImageCommand),
    Ports(commands::ListPortCommand),
    #[clap(alias = "info")]
    Show(commands::InstanceShowCommand),
    #[clap(alias = "config")]
    Modify(InstanceModifyCommand),
    Console(commands::InstanceConsoleCommand),
    Ssh(commands::InstanceSshCommand),
    Scp(commands::InstanceScpCommand),
    Start(commands::InstanceStartCommand),
    Stop(commands::InstanceStopCommand),
    Restart(commands::InstanceRestartCommand),
    Rename(InstanceRenameCommand),
    Clone(InstanceCloneCommand),
    #[clap(alias = "rm", alias = "del")]
    Delete(commands::DeleteInstanceCommand),
    /// Image subcommands
    #[command(subcommand)]
    Image(commands::ImageCommands),
    Prune(commands::PruneCommand),

    /// Network subcommands (Deprecated)
    #[command(subcommand, hide = true)]
    Net(commands::NetworkCommands),
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
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
pub struct CommandDispatcher {
    #[command(subcommand)]
    pub command: Commands,

    #[clap(flatten)]
    global: GlobalOptions,
}

impl CommandDispatcher {
    pub fn dispatch(self, console: &mut dyn Console) -> Result<(), Error> {
        let verbosity = commands::Verbosity::new(self.global.verbose, self.global.quiet);
        console.set_verbosity(verbosity);
        let env = EnvironmentFactory::create_env()?;
        let image_dao = ImageDao::new(&env)?;
        let instance_dao = InstanceDao::new(&env)?;

        match &self.command {
            Commands::Run(cmd) => cmd.run(console, &image_dao, &instance_dao, verbosity),
            Commands::Ls(cmd) => cmd.run(console, &instance_dao),
            Commands::Images(cmd) => cmd.run(console, &env),
            Commands::Ports(cmd) => cmd.run(console, &instance_dao),
            Commands::Add(cmd) => cmd.run(console, &image_dao, &instance_dao),
            Commands::Modify(cmd) => cmd.run(&instance_dao),
            Commands::Clone(cmd) => cmd.run(&instance_dao),
            Commands::Rename(cmd) => cmd.run(&instance_dao),
            Commands::Show(cmd) => cmd.run(console, &instance_dao),
            Commands::Start(cmd) => cmd.run(&instance_dao, verbosity),
            Commands::Stop(cmd) => cmd.run(&instance_dao, verbosity),
            Commands::Restart(cmd) => cmd.run(&instance_dao, verbosity),
            Commands::Console(cmd) => cmd.run(&instance_dao, verbosity),
            Commands::Ssh(cmd) => cmd.run(console, &instance_dao, verbosity),
            Commands::Scp(cmd) => cmd.run(&instance_dao, verbosity),
            Commands::Delete(cmd) => cmd.run(&instance_dao, verbosity),
            Commands::Image(command) => command.dispatch(console, &image_dao),
            Commands::Prune(cmd) => cmd.run(&image_dao),
            Commands::Net(command) => command.dispatch(console, &instance_dao),
        }
    }
}
