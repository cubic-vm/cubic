use crate::commands::{self, Verbosity};
use crate::error::Error;
use crate::image::ImageDao;
use crate::instance::InstanceDao;
use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    /// Setup and run a new instance
    Run {
        /// Name of the virtual machine image
        #[clap(short, long)]
        image: String,
        /// Name of the virtual machine instance
        #[clap(short, long)]
        name: String,
        /// Number of CPUs for the virtual machine instance
        #[clap(short, long)]
        cpus: Option<u16>,
        /// Memory size of the virtual machine instance (e.g. 1G for 1 gigabyte)
        #[clap(short, long)]
        mem: Option<String>,
        /// Disk size of the virtual machine instance  (e.g. 10G for 10 gigabytes)
        #[clap(short, long)]
        disk: Option<String>,
        /// Enable verbose logging
        #[clap(short, long, default_value_t = false)]
        verbose: bool,
        /// Reduce logging output
        #[clap(short, long, default_value_t = false)]
        quiet: bool,
    },

    /// List instances
    List,

    /// Get information about an instance
    Info {
        /// Name of the virtual machine instance
        instance: String,
    },

    /// Open the console of an instance
    Console {
        /// Name of the virtual machine instance
        instance: String,
    },

    /// Open a shell in an instance
    Sh {
        /// Enable verbose logging
        #[clap(short, long, default_value_t = false)]
        verbose: bool,
        /// Reduce logging output
        #[clap(short, long, default_value_t = false)]
        quiet: bool,
        /// Name of the virtual machine instance
        instance: String,
    },

    /// Connect to an instance with SSH
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

    /// Copy a file from or to an instance with SCP
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

    /// Instance commands
    #[command(subcommand)]
    Instance(commands::InstanceCommands),

    /// Image commands
    #[command(subcommand)]
    Image(commands::ImageCommands),

    /// Mount commands
    #[command(subcommand)]
    Mount(commands::MountCommands),

    /// Network commands
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

        let image_dao = ImageDao::new()?;
        let instance_dao = InstanceDao::new()?;

        match &command {
            Commands::Run {
                image,
                name,
                cpus,
                mem,
                disk,
                verbose,
                quiet,
            } => commands::run(
                &image_dao,
                &instance_dao,
                image,
                name,
                cpus,
                mem,
                disk,
                Verbosity::new(*verbose, *quiet),
            ),
            Commands::List => commands::InstanceCommands::list_instances(&instance_dao),
            Commands::Info { instance } => commands::info(&instance_dao, instance.clone()),
            Commands::Start {
                qemu_args,
                verbose,
                quiet,
                instances,
            } => commands::start(
                &instance_dao,
                qemu_args,
                Verbosity::new(*verbose, *quiet),
                instances,
            ),
            Commands::Stop {
                instances,
                verbose,
                quiet,
                all,
            } => commands::stop(
                &instance_dao,
                *all,
                Verbosity::new(*verbose, *quiet),
                instances,
            ),
            Commands::Restart {
                verbose,
                quiet,
                instances,
            } => commands::restart(&instance_dao, Verbosity::new(*verbose, *quiet), instances),
            Commands::Console { instance } => commands::console(&instance_dao, instance),
            Commands::Sh {
                verbose,
                quiet,
                instance,
            } => commands::sh(&instance_dao, Verbosity::new(*verbose, *quiet), instance),
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
            Commands::Instance(command) => command.dispatch(&image_dao, &instance_dao),
            Commands::Image(command) => command.dispatch(&image_dao),
            Commands::Mount(command) => command.dispatch(&instance_dao),
            Commands::Net(command) => command.dispatch(&instance_dao),
        }
    }
}
