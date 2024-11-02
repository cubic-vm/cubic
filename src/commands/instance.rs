use crate::error::Error;
use crate::image::ImageDao;
use crate::machine::{Machine, MachineDao, MachineState, USER};
use crate::util;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum InstanceCommands {
    /// List instances
    List,

    /// Add an instance
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

    /// Delete instances
    Del { instances: Vec<String> },

    /// Read and write configuration parameters
    Config {
        instance: String,
        #[clap(short, long)]
        cpus: Option<u16>,
        #[clap(short, long)]
        mem: Option<String>,
        #[clap(short, long)]
        disk: Option<String>,
    },

    /// Clone an instane
    Clone { name: String, new_name: String },

    /// Rename an instance
    Rename { old_name: String, new_name: String },
}

impl InstanceCommands {
    pub fn dispatch(&self, image_dao: &ImageDao, machine_dao: &MachineDao) -> Result<(), Error> {
        match self {
            InstanceCommands::List => InstanceCommands::list_instances(machine_dao),
            InstanceCommands::Add {
                image,
                name,
                cpus,
                mem,
                disk,
            } => {
                InstanceCommands::add_instance(image_dao, machine_dao, image, name, cpus, mem, disk)
            }
            InstanceCommands::Del { instances } => {
                InstanceCommands::delete_instance(machine_dao, instances)
            }
            InstanceCommands::Config {
                instance,
                cpus,
                mem,
                disk,
            } => InstanceCommands::config(machine_dao, instance, cpus, mem, disk),

            InstanceCommands::Clone { name, new_name } => {
                machine_dao.clone(&machine_dao.load(name)?, new_name)?;

                let mut new_machine = machine_dao.load(new_name)?;
                new_machine.ssh_port = util::generate_random_ssh_port();
                machine_dao.store(&new_machine)
            }

            InstanceCommands::Rename { old_name, new_name } => {
                machine_dao.rename(&mut machine_dao.load(old_name)?, new_name)
            }
        }
    }

    pub fn list_instances(machine_dao: &MachineDao) -> Result<(), Error> {
        let machine_names = machine_dao.get_machines();

        println!(
            "{:15}  {: >4}  {: >9}  {: >9}  {:10}",
            "Name", "CPUs", "Memory", "Disk", "State"
        );
        for machine_name in machine_names {
            let machine = machine_dao.load(&machine_name)?;
            println!(
                "{:15}  {: >4}  {: >9}  {: >9}  {:10}",
                machine_name,
                &machine.cpus,
                util::bytes_to_human_readable(machine.mem),
                util::bytes_to_human_readable(machine.disk_capacity),
                match machine_dao.get_state(&machine) {
                    MachineState::Stopped => "STOPPED",
                    MachineState::Starting => "STARTING",
                    MachineState::Running => "RUNNING",
                }
            );
        }

        Result::Ok(())
    }

    pub fn add_instance(
        image_dao: &ImageDao,
        machine_dao: &MachineDao,
        image_name: &str,
        name: &Option<String>,
        cpus: &Option<u16>,
        mem: &Option<String>,
        disk: &Option<String>,
    ) -> Result<(), Error> {
        let image = image_dao.get(image_name)?;
        image_dao.fetch(&image)?;

        if let Option::Some(instance) = name {
            let machine_dir = format!("{}/{instance}", machine_dao.machine_dir);

            if let Some(id) = name {
                if machine_dao.exists(id) {
                    return Result::Err(Error::MachineAlreadyExists(id.to_string()));
                }
            }

            let image_size = image_dao.get_disk_capacity(&image)?;
            let disk_capacity = disk
                .as_ref()
                .map(|size| util::human_readable_to_bytes(size))
                .unwrap_or(Result::Ok(image_size))?;

            image_dao.copy_image(&image, &machine_dir, "machine.img")?;

            let ssh_port = util::generate_random_ssh_port();

            let mut machine = Machine {
                name: instance.clone(),
                user: USER.to_string(),
                cpus: cpus.unwrap_or(1),
                mem: util::human_readable_to_bytes(mem.as_deref().unwrap_or("1G"))?,
                disk_capacity,
                ssh_port,
                display: false,
                gpu: false,
                mounts: Vec::new(),
            };
            machine_dao.store(&machine)?;
            if disk.is_some() {
                machine_dao.resize(&mut machine, disk_capacity)?;
            }
        }

        Result::Ok(())
    }

    pub fn config(
        machine_dao: &MachineDao,
        instance: &str,
        cpus: &Option<u16>,
        mem: &Option<String>,
        disk: &Option<String>,
    ) -> Result<(), Error> {
        let mut machine = machine_dao.load(instance)?;

        if let Some(cpus) = cpus {
            machine.cpus = *cpus;
        }

        if let Some(mem) = mem {
            machine.mem = util::human_readable_to_bytes(mem)?;
        }

        if let Some(disk) = disk {
            machine_dao.resize(&mut machine, util::human_readable_to_bytes(disk)?)?;
        }

        machine_dao.store(&machine)?;
        println!("cpus:    {}", machine.cpus);
        println!("mem:     {}", util::bytes_to_human_readable(machine.mem));
        println!(
            "disk:    {}",
            util::bytes_to_human_readable(machine.disk_capacity)
        );
        println!("user:    {}", machine.user);
        println!("mounts:");
        for mount in &machine.mounts {
            println!("  - {} => {}", mount.host, mount.guest);
        }
        println!("display: {}", machine.display);
        println!("gpu: {}", machine.gpu);
        println!("ssh-port: {}", machine.ssh_port);
        Result::Ok(())
    }

    pub fn delete_instance(machine_dao: &MachineDao, instances: &Vec<String>) -> Result<(), Error> {
        for instance in instances {
            if !machine_dao.exists(instance) {
                return Result::Err(Error::UnknownMachine(instance.clone()));
            }

            if machine_dao.is_running(&machine_dao.load(instance)?) {
                return Result::Err(Error::MachineNotStopped(instance.to_string()));
            }
        }

        for instance in instances {
            if util::confirm(&format!(
                "Do you really want delete the instance '{instance}'? [y/n]: "
            )) {
                machine_dao.delete(&machine_dao.load(instance)?)?;
                println!("Deleted instance {instance}");
            }
        }

        Result::Ok(())
    }
}
