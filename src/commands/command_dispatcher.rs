use crate::commands::{
    self, InstanceAddCommand, InstanceCloneCommand, InstanceConfigCommand, InstanceInfoCommand,
    InstanceListCommand, InstanceRemoveCommand, InstanceRenameCommand, Verbosity,
};
use crate::env::EnvironmentFactory;
use crate::error::Error;
use crate::image::ImageDao;
use crate::instance::InstanceDao;
use crate::view::{Console, Stdio};
use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    Run(commands::InstanceRunCommand),
    Add(InstanceAddCommand),
    #[clap(alias = "list")]
    Ls(InstanceListCommand),
    #[clap(alias = "del")]
    Rm(InstanceRemoveCommand),
    Info(InstanceInfoCommand),
    Console(commands::InstanceConsoleCommand),
    Ssh(commands::InstanceSshCommand),
    Scp(commands::InstanceScpCommand),
    Start(commands::InstanceStartCommand),

    /// Stop virtual machine instances
    Stop {
        /// Stop all virtual machine instances
        #[clap(short, long, default_value_t = false)]
        all: bool,
        /// Enable verbose logging
        #[clap(short, long, default_value_t = false)]
        verbose: bool,
        /// Reduce logging output
        #[clap(short, long, default_value_t = false)]
        quiet: bool,
        /// Wait for the virtual machine instance to be stopped
        #[clap(short, long, default_value_t = false)]
        wait: bool,
        /// Name of the virtual machine instances to stop
        instances: Vec<String>,
    },

    /// Restart virtual machine instances
    Restart {
        /// Enable verbose logging
        #[clap(short, long, default_value_t = false)]
        verbose: bool,
        /// Reduce logging output
        #[clap(short, long, default_value_t = false)]
        quiet: bool,
        /// Name of the virtual machine instances to restart
        instances: Vec<String>,
    },

    Config(InstanceConfigCommand),
    Rename(InstanceRenameCommand),
    Clone(InstanceCloneCommand),

    /// Image subcommands
    #[command(subcommand)]
    Image(commands::ImageCommands),

    /// Network subcommands
    #[command(subcommand)]
    Net(commands::NetworkCommands),
}

#[derive(Default, Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
pub struct CommandDispatcher {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

impl CommandDispatcher {
    pub fn new() -> Self {
        CommandDispatcher::default()
    }

    pub fn dispatch(self) -> Result<(), Error> {
        let command = Self::parse().command.ok_or(Error::UnknownCommand)?;

        let console: &mut dyn Console = &mut Stdio::new();
        let env = EnvironmentFactory::create_env()?;
        let image_dao = ImageDao::new(&env)?;
        let instance_dao = InstanceDao::new(&env)?;

        match &command {
            Commands::Run(cmd) => cmd.run(&image_dao, &instance_dao),
            Commands::Ls(cmd) => cmd.run(console, &instance_dao),
            Commands::Add(cmd) => cmd.run(&image_dao, &instance_dao),
            Commands::Rm(cmd) => cmd.run(&instance_dao),
            Commands::Clone(cmd) => cmd.run(&instance_dao),
            Commands::Rename(cmd) => cmd.run(&instance_dao),
            Commands::Info(cmd) => cmd.run(console, &instance_dao),
            Commands::Config(cmd) => cmd.run(&instance_dao),
            Commands::Start(cmd) => cmd.run(&instance_dao),
            Commands::Stop {
                instances,
                verbose,
                quiet,
                wait,
                all,
            } => commands::stop(
                &instance_dao,
                *all,
                Verbosity::new(*verbose, *quiet),
                *wait,
                instances,
            ),
            Commands::Restart {
                verbose,
                quiet,
                instances,
            } => commands::restart(&instance_dao, Verbosity::new(*verbose, *quiet), instances),
            Commands::Console(cmd) => cmd.run(&instance_dao),
            Commands::Ssh(cmd) => cmd.run(&instance_dao),
            Commands::Scp(cmd) => cmd.run(&instance_dao),
            Commands::Image(command) => command.dispatch(&image_dao),
            Commands::Net(command) => command.dispatch(&instance_dao),
        }
    }
}
