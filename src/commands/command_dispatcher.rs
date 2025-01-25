use crate::commands::{self, Verbosity};
use crate::error::Error;
use crate::image::ImageDao;
use crate::instance::InstanceDao;
use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    /// Setup and run a new instance
    Run {
        #[clap(short, long)]
        image: String,
        #[clap(short, long)]
        name: String,
        #[clap(short, long)]
        cpus: Option<u16>,
        #[clap(short, long)]
        mem: Option<String>,
        #[clap(short, long)]
        disk: Option<String>,
        #[clap(short, long, default_value_t = false)]
        verbose: bool,
        #[clap(short, long, default_value_t = false)]
        quiet: bool,
    },

    /// List instances
    List,

    /// Get information about an instance
    Info { instance: String },

    /// Open the console of an instance
    Console {
        /// Name of the instance
        instance: String,
    },

    /// Open a shell in an instance
    Sh {
        #[clap(short, long, default_value_t = false)]
        verbose: bool,
        #[clap(short, long, default_value_t = false)]
        quiet: bool,
        instance: String,
    },

    /// Connect to an instance with SSH
    Ssh {
        instance: String,
        /// Forward X over SSH
        #[clap(short = 'X', default_value_t = false)]
        xforward: bool,
        #[clap(short, long, default_value_t = false)]
        verbose: bool,
        #[clap(short, long, default_value_t = false)]
        quiet: bool,
        #[clap(long)]
        ssh_args: Option<String>,
        cmd: Option<String>,
    },

    /// Copy a file from or to an instance with SCP
    Scp {
        from: String,
        to: String,
        #[clap(short, long, default_value_t = false)]
        verbose: bool,
        #[clap(short, long, default_value_t = false)]
        quiet: bool,
        #[clap(long)]
        scp_args: Option<String>,
    },

    /// Start instances
    Start {
        #[clap(long)]
        qemu_args: Option<String>,
        #[clap(short, long, default_value_t = false)]
        verbose: bool,
        #[clap(short, long, default_value_t = false)]
        quiet: bool,
        instances: Vec<String>,
    },

    /// Stop instances
    Stop {
        #[clap(short, long, default_value_t = false)]
        all: bool,
        #[clap(short, long, default_value_t = false)]
        verbose: bool,
        #[clap(short, long, default_value_t = false)]
        quiet: bool,
        instances: Vec<String>,
    },

    /// Restart instances
    Restart {
        #[clap(short, long, default_value_t = false)]
        verbose: bool,
        #[clap(short, long, default_value_t = false)]
        quiet: bool,
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
