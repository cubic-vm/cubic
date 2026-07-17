use crate::actions::CreateInstanceAction;
use crate::commands::{
    self, Command, Context,
    image::{fetch_image, fetch_image_info},
};
use crate::error::{Error, Result};
use crate::fs::FS;
use crate::models::{DataSize, ImageName, Instance, PortForward, ResourceAllocator};
use crate::ssh_cmd::PortChecker;
use crate::view::Console;
use crate::view::Spinner;
use clap::{ArgAction, Parser};
use std::sync::{Arc, Mutex};

pub const DEFAULT_DISK_SIZE: &str = "100G";

/// Create VM instances
///
/// This command only creates the VM instance. Use cubic start <instance> to power
/// it on and cubic ssh <instance> to connect to it.
///
/// Examples:
///
///   Create a VM instance with 8 vCPUs, 10G of RAM, 200G of storage:
///   $ cubic create example1 --cpus 8 --memory 10G --disk 200G -i debian:trixie
///
///   Create a VM instance and forward the instance's HTTP port to the host port 8000:
///   $ cubic create example2 --port 8000:80 -i ubuntu:noble
///
///   Create a VM instance and forward the instance's DNS port to the host port 5353:
///   $ cubic create example3 --port 5353:53/udp -i ubuntu:noble
///
///   Create a VM instance with multiple port forwarding rules:
///   $ cubic create example4 -p 8000:80/tcp -p 5353:53/udp -i ubuntu:noble
///
///   Create a VM instance and install Vim:
///   $ cubic create example5 -e "sudo apt install -y vim" -i ubuntu:noble
///
///   Create a VM instance without network access:
///   $ cubic create example6 --isolate ubuntu:noble
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct CreateCommand {
    #[clap(flatten)]
    pub instance_name: commands::InstanceArg,
    /// VM image name (e.g. 'debian:trixie')
    #[clap(short, long)]
    image: ImageName,
    /// Username (default: 'cubic')
    #[clap(short, long)]
    user: Option<String>,
    /// Number of vCPUs for the VM instance (default: derived from host resources)
    #[clap(short, long)]
    cpus: Option<u16>,
    /// Memory amount of the VM instance (default: derived from host resources)
    #[clap(alias = "mem", short, long)]
    memory: Option<DataSize>,
    /// Disk size of the VM instance
    #[clap(short, long, default_value = DEFAULT_DISK_SIZE)]
    disk: DataSize,
    /// Forward ports from guest to host (e.g. -p 8000:80 or -p 9000:90/tcp)
    #[clap(short, long)]
    port: Vec<PortForward>,
    /// Execute a command once on the first boot (e.g. "sudo apt install ...")
    #[clap(short, long)]
    execute: Option<String>,
    /// Isolate the VM instance from network
    #[clap(long, action = ArgAction::SetTrue)]
    isolate: bool,
}

impl Command for CreateCommand {
    fn run(&self, console: &mut Console<'_>, context: &Context) -> Result<()> {
        let env = context.get_env();
        let instance_store = context.get_instance_store();

        if instance_store.exists(self.instance_name.value.as_str()) {
            return Err(Error::InstanceAlreadyExists(
                self.instance_name.value.to_string(),
            ));
        }

        // Fetch image
        let image = &fetch_image_info(console, env, &self.image)?;
        fetch_image(console, env, context.get_image_store(), image)?;

        console.play(Arc::new(Mutex::new(Spinner::new(format!(
            "Creating {}",
            self.instance_name.value
        )))));
        let ssh_port = PortChecker::new().get_new_port()?;

        let (default_cpus, default_mem) =
            ResourceAllocator::read_from_host().get_default_resources();

        let instance = Instance {
            name: self.instance_name.value.to_string(),
            arch: image.arch,
            user: self
                .user
                .as_deref()
                .unwrap_or(context.get_env().get_username())
                .to_string(),
            cpus: self.cpus.unwrap_or(default_cpus),
            mem: self.memory.clone().unwrap_or(default_mem),
            disk_capacity: self.disk.clone(),
            ssh_port,
            hostfwd: self.port.clone(),
            execute: self.execute.clone(),
            isolate: self.isolate,
            ..Instance::default()
        };

        console.debug(&format!(
            "Resolved instance '{}': {} vCPUs, {} memory, {} disk, ssh_port={}",
            instance.name,
            instance.cpus,
            instance.mem.to_size(),
            instance.disk_capacity.to_size(),
            instance.ssh_port,
        ));

        let image_path = &env.get_image_file(&image.to_file_name());
        CreateInstanceAction::new().run(context, &FS::new(), image_path, instance)?;

        console.stop();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::ImageStoreMock;
    use crate::instance::InstanceStoreMock;
    use crate::models::Environment;
    use crate::platform::SystemMock;
    use std::rc::Rc;

    #[test]
    fn test_create_rejects_existing_instance_name() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let env = Environment::new(
            "cubic".to_string(),
            String::new(),
            String::new(),
            String::new(),
        );
        let context = Context::new(
            Rc::new(SystemMock::new()),
            env,
            Box::new(ImageStoreMock::default()),
            Box::new(InstanceStoreMock::new(vec![Instance {
                name: "test".to_string(),
                ..Instance::default()
            }])),
        );

        let result = CreateCommand::try_parse_from(["create", "test", "-i", "debian:bookworm"])
            .unwrap()
            .run(console, &context);

        assert!(matches!(
            result,
            Err(Error::InstanceAlreadyExists(ref name)) if name == "test"
        ));
    }
}
