mod commands;
mod error;
mod image;
mod machine;
mod util;

use clap::{Parser, Subcommand};
use error::Error;
use image::ImageDao;
use machine::MachineDao;

#[derive(Subcommand)]
enum Commands {
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
    List { name: Option<String> },

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

    /// Connect to a machine with SSH
    Ssh {
        instance: String,
        cmd: Option<String>,
    },

    /// Copy a file from or to a machine with SCP
    Scp { from: String, to: String },
}

#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

fn dispatch(command: &Commands) -> Result<(), Error> {
    let image_dao = ImageDao::new()?;
    let machine_dao = MachineDao::new()?;

    match command {
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
        Commands::List { name } => commands::list(&image_dao, &machine_dao, name),
        Commands::Start { console, ids } => commands::start(&machine_dao, *console, ids),
        Commands::Stop { ids, all } => commands::stop(&machine_dao, ids, *all),
        Commands::Restart { console, ids } => commands::restart(&machine_dao, *console, ids),
        Commands::Ssh { instance, cmd } => commands::ssh(&machine_dao, instance, cmd),
        Commands::Scp { from, to } => commands::scp(&machine_dao, from, to),
    }
}

fn main() {
    let result = Cli::parse()
        .command
        .ok_or(Error::UnknownCommand)
        .and_then(|command| dispatch(&command));

    if let Result::Err(error) = result {
        print!("ERROR: ");
        match error {
            Error::UnknownCommand => println!("Unknown command"),
            Error::UnknownMachine(machine) => println!("Unknown machine '{machine}'"),
            Error::MachineNotStopped(name) => println!("Machine '{name}' is not stopped"),
            Error::Start(machine) => println!("Failed to start machine '{machine}'"),
            Error::Stop(machine) => println!("Failed to stop machine '{machine}'"),
            Error::MachineAlreadyExists(id) => println!("Machine with name '{id}' already exists"),
            Error::Io(e) => println!("{}", e),
            Error::UnknownImage(name) => println!("Unknown image name {name}"),
            Error::MissingSshKey => println!("Could not find any ssh keys. Please create a ssh key to access the virtual machine"),
            Error::InvalidImageName(name) => println!("Invalid image name: {name}"),
            Error::UnsetHomeVar => println!("Environment variable 'HOME' is not defined"),
            Error::InvalidOption(option) => println!("'{option}' is not a valid option"),
            Error::CannotCopyFile(from, to) => println!("Cannot copy file from '{from}' to '{to}'"),
            Error::CannotCopyDir(from, to) => println!("Cannot copy directory from '{from}' to '{to}'"),
            Error::CannotCreateFile(path) => println!("Cannot create file '{path}'"),
            Error::CannotOpenFile(path) => println!("Cannot open file '{path}'"),
            Error::CannotCreateDir(path) => println!("Cannot create directory '{path}'"),
            Error::CannotAccessDir(path) => println!("Cannot access directory '{path}'"),
            Error::CannotWriteDir(path) => println!("Cannot write directory '{path}'"),
            Error::CannotWriteFile(path) => println!("Cannot write file '{path}'"),
            Error::CannotRemoveFile(path) => println!("Cannot write file '{path}'"),
            Error::CannotParseFile(path) => println!("Cannot parse file '{path}'"),
            Error::InvalidSshTarget(name) => println!("Invalid SSH target '{name}'"),
            Error::UserDataCreationFailed(name) => println!("Failed to create user data for machine '{name}'"),
            Error::CannotParseSize(size) => println!("Invalid data size format '{size}'"),
            Error::CannotShrinkDisk(name) => println!("Cannot shrink the disk of the machine '{name}'"),
            Error::ImageDownloadFailed(name) => println!("Failed to download image: '{name}'"),
            Error::GetCapacityFailed(path) => println!("Failed to get capacity from image: '{path}'"),
        }
    }
}
