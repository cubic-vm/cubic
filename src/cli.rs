use crate::commands;
use crate::error::Error;
use crate::image::ImageDao;
use crate::machine::MachineDao;
use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    /// Add and start a new machine
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

    /// Add an image or a machine
    Add {
        #[clap(short, long)]
        image: String,
        #[clap(short, long)]
        name: Option<String>,
        #[clap(short, long)]
        cpus: Option<u16>,
        #[clap(short, long)]
        mem: Option<String>,
        #[clap(short, long)]
        disk: Option<String>,
    },

    /// Delete images and machines
    Delete { ids: Vec<String> },

    /// Clone a machine
    Clone { name: String, new_name: String },

    /// Rename a machine
    Rename { old_name: String, new_name: String },

    /// Read and write configuration parameters
    Config {
        instance: String,
        #[clap(short, long)]
        cpus: Option<u16>,
        #[clap(short, long)]
        mem: Option<String>,
        #[clap(short, long)]
        disk: Option<String>,
        #[clap(short, long)]
        sandbox: Option<bool>,
    },

    /// List images and machines
    List {
        #[clap(short, long, default_value_t = false)]
        all: bool,
        name: Option<String>,
    },

    /// Start machines
    Start {
        #[clap(short, long, default_value_t = false)]
        console: bool,
        ids: Vec<String>,
    },

    /// Stop machines
    Stop {
        #[clap(short, long, default_value_t = false)]
        all: bool,
        ids: Vec<String>,
    },

    /// Restart a machine
    Restart {
        #[clap(short, long, default_value_t = false)]
        console: bool,
        ids: Vec<String>,
    },

    /// Open a shell in the machine
    Sh {
        #[clap(short, long, default_value_t = false)]
        console: bool,
        instance: String,
    },

    /// Connect to a machine with SSH
    Ssh {
        instance: String,
        #[clap(long)]
        ssh_args: Option<String>,
        cmd: Option<String>,
    },

    /// Copy a file from or to a machine with SCP
    Scp { from: String, to: String },

    /// Mount host directory to guest
    Mount {
        name: String,
        host: String,
        guest: String,
    },

    /// Unmount guest directory
    Umount { name: String, guest: String },
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
        Commands::Add {
            image,
            name,
            cpus,
            mem,
            disk,
        } => commands::add(&image_dao, &machine_dao, image, name, cpus, mem, disk),
        Commands::Delete { ids } => commands::delete(&image_dao, &machine_dao, ids),
        Commands::Clone { name, new_name } => commands::clone(&machine_dao, name, new_name),
        Commands::Rename { old_name, new_name } => {
            machine_dao.rename(&mut machine_dao.load(old_name)?, new_name)
        }
        Commands::Config {
            instance,
            cpus,
            mem,
            disk,
            sandbox,
        } => commands::config(&machine_dao, instance, cpus, mem, disk, sandbox),
        Commands::List { name, all } => commands::list(&image_dao, &machine_dao, name, *all),
        Commands::Start { console, ids } => commands::start(&machine_dao, *console, ids),
        Commands::Stop { ids, all } => commands::stop(&machine_dao, ids, *all),
        Commands::Restart { console, ids } => commands::restart(&machine_dao, *console, ids),
        Commands::Sh { console, instance } => commands::sh(&machine_dao, *console, instance),
        Commands::Ssh {
            instance,
            ssh_args,
            cmd,
        } => commands::ssh(&machine_dao, instance, ssh_args, cmd),
        Commands::Scp { from, to } => commands::scp(&machine_dao, from, to),
        Commands::Mount { name, host, guest } => commands::mount(&machine_dao, name, host, guest),
        Commands::Umount { name, guest } => commands::umount(&machine_dao, name, guest),
    }
}
