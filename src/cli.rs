use crate::commands;
use crate::error::Error;
use crate::image::ImageDao;
use crate::machine::MachineDao;
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
    },

    /// List instances
    List,

    /// Get information about an instance
    Info { instance: String },

    /// Open a shell in an instance
    Sh {
        #[clap(short, long, default_value_t = false)]
        console: bool,
        #[clap(short, long, default_value_t = false)]
        verbose: bool,
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
        #[clap(long)]
        scp_args: Option<String>,
    },

    /// Start instances
    Start {
        #[clap(long)]
        qemu_args: Option<String>,
        #[clap(short, long, default_value_t = false)]
        console: bool,
        #[clap(short, long, default_value_t = false)]
        verbose: bool,
        ids: Vec<String>,
    },

    /// Stop instances
    Stop {
        #[clap(short, long, default_value_t = false)]
        all: bool,
        ids: Vec<String>,
    },

    /// Restart instances
    Restart {
        #[clap(short, long, default_value_t = false)]
        console: bool,
        #[clap(short, long, default_value_t = false)]
        verbose: bool,
        ids: Vec<String>,
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
}

#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

pub fn dispatch(command: Commands) -> Result<(), Error> {
    let image_dao = ImageDao::new()?;
    let machine_dao = MachineDao::new()?;

    match &command {
        Commands::Run {
            image,
            name,
            cpus,
            mem,
            disk,
        } => commands::run(&image_dao, &machine_dao, image, name, cpus, mem, disk),
        Commands::List => commands::InstanceCommands::list_instances(&machine_dao),
        Commands::Info { instance } => commands::info(&machine_dao, instance.clone()),
        Commands::Start {
            qemu_args,
            console,
            verbose,
            ids,
        } => commands::start(&machine_dao, qemu_args, *console, *verbose, ids),
        Commands::Stop { ids, all } => commands::stop(&machine_dao, ids, *all),
        Commands::Restart {
            console,
            verbose,
            ids,
        } => commands::restart(&machine_dao, *console, *verbose, ids),
        Commands::Sh {
            console,
            verbose,
            instance,
        } => commands::sh(&machine_dao, *console, *verbose, instance),
        Commands::Ssh {
            instance,
            xforward,
            verbose,
            ssh_args,
            cmd,
        } => commands::ssh(&machine_dao, instance, *xforward, *verbose, ssh_args, cmd),
        Commands::Scp {
            from,
            to,
            verbose,
            scp_args,
        } => commands::scp(&machine_dao, from, to, *verbose, scp_args),
        Commands::Instance(command) => command.dispatch(&image_dao, &machine_dao),
        Commands::Image(command) => command.dispatch(&image_dao),
        Commands::Mount(command) => command.dispatch(&machine_dao),
    }
}
