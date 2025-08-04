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

    /// Open the console of an virtual machine instance
    Console {
        /// Name of the virtual machine instance
        instance: String,
    },

    /// Connect to a virtual machine instance with SSH
    Ssh {
        /// Name of the virtual machine instance
        instance: String,
        /// Forward X over SSH
        #[clap(short = 'X', default_value_t = false)]
        xforward: bool,
        /// Enable verbose logging
        #[clap(short, long, default_value_t = false)]
        verbose: bool,
        /// Reduce logging output
        #[clap(short, long, default_value_t = false)]
        quiet: bool,
        /// Pass additional SSH arguments
        #[clap(long)]
        ssh_args: Option<String>,
        /// Execute a command in the virtual machine
        cmd: Option<String>,
    },

    /// Copy a file from or to a virtual machine instance with SCP
    Scp {
        /// Source of the data to copy
        from: String,
        /// Target of the data to copy
        to: String,
        /// Enable verbose logging
        #[clap(short, long, default_value_t = false)]
        verbose: bool,
        /// Reduce logging output
        #[clap(short, long, default_value_t = false)]
        quiet: bool,
        /// Pass additional SCP arguments
        #[clap(long)]
        scp_args: Option<String>,
    },

    /// Start virtual machine instances
    Start {
        /// Pass additional QEMU arguments
        #[clap(long)]
        qemu_args: Option<String>,
        /// Enable verbose logging
        #[clap(short, long, default_value_t = false)]
        verbose: bool,
        /// Reduce logging output
        #[clap(short, long, default_value_t = false)]
        quiet: bool,
        /// Wait for the virtual machine instance to be started
        #[clap(short, long, default_value_t = false)]
        wait: bool,
        /// Name of the virtual machine instances to start
        instances: Vec<String>,
    },

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
            Commands::Start {
                qemu_args,
                verbose,
                quiet,
                wait,
                instances,
            } => commands::start(
                &instance_dao,
                qemu_args,
                Verbosity::new(*verbose, *quiet),
                *wait,
                instances,
            ),
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
            Commands::Console { instance } => commands::console(&instance_dao, instance),
            Commands::Ssh {
                instance,
                xforward,
                verbose,
                quiet,
                ssh_args,
                cmd,
            } => commands::ssh(
                &instance_dao,
                instance,
                *xforward,
                Verbosity::new(*verbose, *quiet),
                ssh_args,
                cmd,
            ),
            Commands::Scp {
                from,
                to,
                verbose,
                quiet,
                scp_args,
            } => commands::scp(
                &instance_dao,
                from,
                to,
                Verbosity::new(*verbose, *quiet),
                scp_args,
            ),
            Commands::Image(command) => command.dispatch(&image_dao),
            Commands::Net(command) => command.dispatch(&instance_dao),
        }
    }
}
