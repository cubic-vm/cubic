use crate::error::Error;
use crate::image::ImageDao;
use crate::instance::{Instance, InstanceDao, InstanceState, USER};
use crate::util;
use crate::view::{Alignment, TableView};
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
    pub fn dispatch(&self, image_dao: &ImageDao, instance_dao: &InstanceDao) -> Result<(), Error> {
        match self {
            InstanceCommands::List => InstanceCommands::list_instances(instance_dao),
            InstanceCommands::Add {
                image,
                name,
                cpus,
                mem,
                disk,
            } => InstanceCommands::add_instance(
                image_dao,
                instance_dao,
                image,
                name,
                cpus,
                mem,
                disk,
            ),
            InstanceCommands::Del { instances } => {
                InstanceCommands::delete_instance(instance_dao, instances)
            }
            InstanceCommands::Config {
                instance,
                cpus,
                mem,
                disk,
            } => InstanceCommands::config(instance_dao, instance, cpus, mem, disk),

            InstanceCommands::Clone { name, new_name } => {
                instance_dao.clone(&instance_dao.load(name)?, new_name)?;

                let mut new_instance = instance_dao.load(new_name)?;
                new_instance.ssh_port = util::generate_random_ssh_port();
                instance_dao.store(&new_instance)
            }

            InstanceCommands::Rename { old_name, new_name } => {
                instance_dao.rename(&mut instance_dao.load(old_name)?, new_name)
            }
        }
    }

    pub fn list_instances(instance_dao: &InstanceDao) -> Result<(), Error> {
        let instance_names = instance_dao.get_instances();

        let mut view = TableView::new();
        view.add_row()
            .add("Name", Alignment::Left)
            .add("CPUs", Alignment::Right)
            .add("Memory", Alignment::Right)
            .add("Disk", Alignment::Right)
            .add("State", Alignment::Left);

        for instance_name in &instance_names {
            let instance = instance_dao.load(instance_name)?;
            view.add_row()
                .add(instance_name, Alignment::Left)
                .add(&instance.cpus.to_string(), Alignment::Right)
                .add(
                    &util::bytes_to_human_readable(instance.mem),
                    Alignment::Right,
                )
                .add(
                    &util::bytes_to_human_readable(instance.disk_capacity),
                    Alignment::Right,
                )
                .add(
                    match instance_dao.get_state(&instance) {
                        InstanceState::Stopped => "STOPPED",
                        InstanceState::Starting => "STARTING",
                        InstanceState::Running => "RUNNING",
                    },
                    Alignment::Left,
                );
        }
        view.print();
        Result::Ok(())
    }

    pub fn add_instance(
        image_dao: &ImageDao,
        instance_dao: &InstanceDao,
        image_name: &str,
        name: &Option<String>,
        cpus: &Option<u16>,
        mem: &Option<String>,
        disk: &Option<String>,
    ) -> Result<(), Error> {
        let image = image_dao.get(image_name)?;
        image_dao.fetch(&image)?;

        if let Option::Some(instance) = name {
            let instance_dir = format!("{}/{instance}", instance_dao.instance_dir);

            if let Some(id) = name {
                if instance_dao.exists(id) {
                    return Result::Err(Error::InstanceAlreadyExists(id.to_string()));
                }
            }

            let image_size = image_dao.get_disk_capacity(&image)?;
            let disk_capacity = disk
                .as_ref()
                .map(|size| util::human_readable_to_bytes(size))
                .unwrap_or(Result::Ok(image_size))?;

            image_dao.copy_image(&image, &instance_dir, "machine.img")?;

            let ssh_port = util::generate_random_ssh_port();

            let mut instance = Instance {
                name: instance.clone(),
                user: USER.to_string(),
                cpus: cpus.unwrap_or(1),
                mem: util::human_readable_to_bytes(mem.as_deref().unwrap_or("1G"))?,
                disk_capacity,
                ssh_port,
                display: false,
                gpu: false,
                mounts: Vec::new(),
                hostfwd: Vec::new(),
            };
            instance_dao.store(&instance)?;
            if disk.is_some() {
                instance_dao.resize(&mut instance, disk_capacity)?;
            }
        }

        Result::Ok(())
    }

    pub fn config(
        instance_dao: &InstanceDao,
        instance: &str,
        cpus: &Option<u16>,
        mem: &Option<String>,
        disk: &Option<String>,
    ) -> Result<(), Error> {
        let mut instance = instance_dao.load(instance)?;

        if let Some(cpus) = cpus {
            instance.cpus = *cpus;
        }

        if let Some(mem) = mem {
            instance.mem = util::human_readable_to_bytes(mem)?;
        }

        if let Some(disk) = disk {
            instance_dao.resize(&mut instance, util::human_readable_to_bytes(disk)?)?;
        }

        instance_dao.store(&instance)?;
        Result::Ok(())
    }

    pub fn delete_instance(
        instance_dao: &InstanceDao,
        instances: &Vec<String>,
    ) -> Result<(), Error> {
        for instance in instances {
            if !instance_dao.exists(instance) {
                return Result::Err(Error::UnknownInstance(instance.clone()));
            }

            if instance_dao.is_running(&instance_dao.load(instance)?) {
                return Result::Err(Error::InstanceNotStopped(instance.to_string()));
            }
        }

        for instance in instances {
            if util::confirm(&format!(
                "Do you really want delete the instance '{instance}'? [y/n]: "
            )) {
                instance_dao.delete(&instance_dao.load(instance)?)?;
                println!("Deleted instance {instance}");
            }
        }

        Result::Ok(())
    }
}
