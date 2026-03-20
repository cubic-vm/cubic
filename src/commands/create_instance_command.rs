use crate::actions::CreateInstanceAction;
use crate::commands::{
    Command,
    image::{fetch_image, fetch_image_info},
};
use crate::env::Environment;
use crate::error::{Error, Result};
use crate::fs::FS;
use crate::image::{ImageName, ImageStore};
use crate::instance::{Instance, InstanceName, InstanceStore, PortForward};
use crate::model::DataSize;
use crate::ssh_cmd::PortChecker;
use crate::view::Console;
use crate::view::SpinnerView;
use clap::{ArgAction, Parser};

pub const DEFAULT_CPU_COUNT: u16 = 4;
pub const DEFAULT_MEM_SIZE: &str = "4G";
pub const DEFAULT_DISK_SIZE: &str = "100G";

/// Create VM instances
///
/// This command only creates the VM instance. Use cubic start <instance> to power
/// it on and cubic ssh <instance> to connect to it.
///
/// Examples:
///
///   Create a VM instance with 8 vCPUs, 10G of RAM, 200G of storage:
///   $ cubic create example1 --cpus 8 --mem 10G --disk 200G -i debian:trixie
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
pub struct CreateInstanceCommand {
    /// VM instance name (e.g. 'my-instance')
    pub instance_name: InstanceName,
    /// VM image name (e.g. 'debian:trixie')
    #[clap(short, long)]
    image: ImageName,
    /// Username (default: 'cubic')
    #[clap(short, long, default_value = "cubic")]
    user: String,
    /// Number of vCPUs for the VM instance
    #[clap(short, long, default_value_t = DEFAULT_CPU_COUNT)]
    cpus: u16,
    /// Memory amount of the VM instance
    #[clap(short, long, default_value = DEFAULT_MEM_SIZE)]
    mem: DataSize,
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

impl Command for CreateInstanceCommand {
    fn run(
        &self,
        _console: &mut dyn Console,
        env: &Environment,
        image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<()> {
        if instance_store.exists(self.instance_name.as_str()) {
            return Result::Err(Error::InstanceAlreadyExists(self.instance_name.to_string()));
        }

        // Fetch image
        let image = &fetch_image_info(env, &self.image)?;
        fetch_image(env, image_store, image)?;

        let mut create_spinner = SpinnerView::new("Creating virtual machine instance".to_string());

        let instance = Instance {
            name: self.instance_name.to_string(),
            arch: image.arch,
            user: self.user.to_string(),
            cpus: self.cpus,
            mem: self.mem.clone(),
            disk_capacity: self.disk.clone(),
            ssh_port: PortChecker::new().get_new_port(),
            hostfwd: self.port.clone(),
            execute: self.execute.clone(),
            isolate: self.isolate,
            ..Instance::default()
        };
        CreateInstanceAction::new().run(env, &FS::new(), instance_store, image, instance)?;

        create_spinner.stop();
        Result::Ok(())
    }
}
