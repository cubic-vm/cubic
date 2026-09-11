use crate::actions::CreateInstanceAction;
use crate::commands::{
    self, Command, Context,
    image::{fetch_image, fetch_image_info},
};
use crate::error::{Error, Result};
use crate::models::{
    Arch, DataSize, Environment, ImageName, Instance, LOW_DISK_SPACE_WARNING, PortForward,
    ResourceAllocator, Template, UserName,
};
use crate::platform::System;
use crate::view::{Console, Spinner};
use clap::{ArgAction, Parser};
use std::path::Path;
use std::sync::Arc;

/// The disk size of a new VM instance, 100 GiB.
pub const DEFAULT_DISK_SIZE: DataSize = DataSize::new(100 * 1024_usize.pow(3));

/// Create a VM instance
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
///   $ cubic create example2 --port 8000:80 -i ubuntu
///
///   Create a VM instance and forward the instance's DNS port to the host port 5353:
///   $ cubic create example3 --port 5353:53/udp -i ubuntu
///
///   Create a VM instance with multiple port forwarding rules:
///   $ cubic create example4 -p 8000:80/tcp -p 5353:53/udp -i ubuntu:latest
///
///   Create a VM instance and install Vim:
///   $ cubic create example5 -e "sudo apt install -y vim" -i ubuntu
///
///   Create a VM instance without network access:
///   $ cubic create example6 --isolate -i ubuntu
///
///   Create a VM instance from a template (command line arguments override the template):
///   $ cubic create example7 --template ./my-template.toml
///
///   Create a VM instance with network access from a template that isolates it:
///   $ cubic create example8 --template ./my-template.toml --no-isolate
///
///   Every distribution has the tags latest and stable. The tag latest is the
///   newest release and the tag stable is the newest long term release. A plain
///   name is a shortcut for stable, so --image ubuntu gives you the last LTS.
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct CreateCommand {
    #[clap(flatten)]
    pub instance_name: commands::InstanceArg,
    /// Template file with default values (e.g. --template ./my-template.toml)
    #[clap(short, long)]
    template: Option<String>,
    /// VM image name (e.g. 'debian:trixie', 'debian:latest' or 'debian')
    #[clap(short, long)]
    image: Option<ImageName>,
    /// Username of the guest account (default: your user name on the host)
    #[clap(short, long)]
    user: Option<UserName>,
    /// Number of vCPUs for the VM instance (default: derived from host resources)
    #[clap(short, long)]
    cpus: Option<u16>,
    /// Memory amount of the VM instance (default: derived from host resources)
    #[clap(alias = "mem", short, long)]
    memory: Option<DataSize>,
    /// Disk size of the VM instance (default: 100G)
    #[clap(short, long)]
    disk: Option<DataSize>,
    /// Forward ports from guest to host (e.g. -p 8000:80 or -p 9000:90/tcp)
    #[clap(short, long)]
    port: Vec<PortForward>,
    /// Execute a command once on the first boot (repeatable, e.g. -e "sudo apt install ...")
    #[clap(short, long)]
    execute: Vec<String>,
    /// Isolate the VM instance from network
    #[clap(long, action = ArgAction::SetTrue, conflicts_with = "no_isolate")]
    isolate: bool,
    /// Connect the VM instance to the network, even if the template isolates it
    #[clap(long, action = ArgAction::SetTrue)]
    no_isolate: bool,
}

impl CreateCommand {
    /// Reads the template, which provides default values that any command line
    /// argument overrides.
    fn read_template(&self, system: &dyn System) -> Result<Option<Template>> {
        self.template
            .as_deref()
            .map(|path| Template::parse(&system.read_file_to_string(Path::new(path))?))
            .transpose()
    }

    fn resolve_image(&self, template: Option<&Template>) -> Result<ImageName> {
        self.image
            .clone()
            .or_else(|| template.and_then(|t| t.image.clone()))
            .ok_or(Error::MissingImage)
    }

    fn build_instance(
        &self,
        template: Option<&Template>,
        env: &Environment,
        arch: Arch,
        ssh_port: u16,
        default_cpus: u16,
        default_mem: DataSize,
    ) -> Instance {
        // A command list is joined so the commands run in order and stop at the first failure.
        let commands = if self.execute.is_empty() {
            template.map(|t| t.run.clone()).unwrap_or_default()
        } else {
            self.execute.clone()
        };

        Instance {
            name: self.instance_name.value.to_string(),
            arch,
            user: self
                .user
                .clone()
                .or_else(|| template.and_then(|t| t.user.clone()))
                .unwrap_or_else(|| env.get_username().clone()),
            cpus: self
                .cpus
                .or_else(|| template.and_then(|t| t.cpus))
                .unwrap_or(default_cpus),
            mem: self
                .memory
                .clone()
                .or_else(|| template.and_then(|t| t.memory.clone()))
                .unwrap_or(default_mem),
            disk_capacity: self
                .disk
                .clone()
                .or_else(|| template.and_then(|t| t.disk.clone()))
                .unwrap_or(DEFAULT_DISK_SIZE),
            ssh_port,
            hostfwd: if self.port.is_empty() {
                template.map(|t| t.ports.clone()).unwrap_or_default()
            } else {
                self.port.clone()
            },
            execute: (!commands.is_empty()).then(|| commands.join(" && ")),
            isolate: !self.no_isolate
                && (self.isolate || template.and_then(|t| t.isolate).unwrap_or(false)),
            ..Instance::default()
        }
    }
}

impl CreateCommand {
    pub async fn create(
        &self,
        console: &Arc<Console>,
        context: &Context,
        auto_remove: bool,
    ) -> Result<()> {
        let env = context.get_env();
        let instance_store = context.get_instance_store();

        instance_store.claim_name(self.instance_name.value.as_str())?;

        if ResourceAllocator::is_disk_space_low(context.get_system(), env) {
            console.warn(LOW_DISK_SPACE_WARNING);
        }

        let template = self.read_template(context.get_system())?;
        let image_name = self.resolve_image(template.as_ref())?;

        // Fetch image
        let image = &fetch_image_info(console, context.get_system(), env, &image_name).await?;
        fetch_image(console, context.get_system(), env, image).await?;

        let text = format!("Creating {}", self.instance_name.value);
        let _spinner = Spinner::new(Arc::clone(console), text);
        let ssh_port = context.get_system().bind_port()?;

        let (default_cpus, default_mem) =
            ResourceAllocator::read_from_host(context.get_system()).get_default_resources();

        let instance = self.build_instance(
            template.as_ref(),
            env,
            image.arch,
            ssh_port,
            default_cpus,
            default_mem,
        );

        console.debug(&format!(
            "Resolved instance '{}': {} vCPUs, {} memory, {} disk, ssh_port={}",
            instance.name,
            instance.cpus,
            instance.mem.to_size(),
            instance.disk_capacity.to_size(),
            instance.ssh_port,
        ));

        let image_path = &env.get_image_file(&image.to_file_name());
        CreateInstanceAction::new().run(context, image_path, instance, auto_remove)?;

        Ok(())
    }
}

impl Command for CreateCommand {
    async fn run(&self, console: &Arc<Console>, context: &Context) -> Result<u8> {
        self.create(console, context, false).await?;
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::InstanceStoreMock;
    use crate::platform::SystemMock;
    use std::str::FromStr;
    use std::sync::Arc;

    const GIB: usize = 1024_usize.pow(3);

    #[tokio::test]
    async fn test_create_rejects_existing_instance_name() {
        let system = SystemMock::new();
        let console = &Console::new(Arc::new(system));
        let env = Environment::new(
            UserName::from_str("cubic").unwrap(),
            String::new(),
            String::new(),
        );
        let context = Context::new(
            Arc::new(SystemMock::new()),
            env,
            Box::new(InstanceStoreMock::new(vec![Instance {
                name: "test".to_string(),
                ..Instance::default()
            }])),
        );

        let result = CreateCommand::try_parse_from(["create", "test", "-i", "debian:bookworm"])
            .unwrap()
            .run(console, &context)
            .await;

        assert!(matches!(
            result,
            Err(Error::InstanceAlreadyExists(ref name)) if name == "test"
        ));
    }

    fn build_env() -> Environment {
        Environment::new(
            UserName::from_str("host-user").unwrap(),
            String::new(),
            String::new(),
        )
    }

    fn build_instance_from(args: &[&str], template: Option<&Template>) -> Instance {
        CreateCommand::try_parse_from(args).unwrap().build_instance(
            template,
            &build_env(),
            Arch::AMD64,
            22,
            2,
            DataSize::new(GIB),
        )
    }

    fn build_template() -> Template {
        Template::parse(
            r#"
version = 1
image = "debian:trixie"
user = "john"
cpus = 4
memory = "4G"
disk = "200G"
isolate = true
ports = ["8000:80"]
run = ["sudo apt update", "sudo apt install vim"]
"#,
        )
        .unwrap()
    }

    #[test]
    fn test_template_fills_values_that_are_not_given_on_the_command_line() {
        let template = build_template();
        let instance = build_instance_from(&["create", "web"], Some(&template));

        assert_eq!(instance.user.as_str(), "john");
        assert_eq!(instance.cpus, 4);
        assert_eq!(instance.mem.get_bytes(), 4 * GIB);
        assert_eq!(instance.disk_capacity.get_bytes(), 200 * GIB);
        assert_eq!(instance.hostfwd.len(), 1);
        assert!(instance.isolate);
        assert_eq!(
            instance.execute,
            Some("sudo apt update && sudo apt install vim".to_string())
        );
    }

    #[test]
    fn test_command_line_arguments_override_the_template() {
        let template = build_template();
        let instance = build_instance_from(
            &[
                "create",
                "web",
                "--user",
                "janne",
                "--cpus",
                "8",
                "--memory",
                "2G",
                "--disk",
                "50G",
                "--port",
                "9000:90",
                "--execute",
                "echo hello",
                "--no-isolate",
            ],
            Some(&template),
        );

        assert_eq!(instance.user.as_str(), "janne");
        assert_eq!(instance.cpus, 8);
        assert_eq!(instance.mem.get_bytes(), 2 * GIB);
        assert_eq!(instance.disk_capacity.get_bytes(), 50 * GIB);
        assert_eq!(instance.hostfwd.len(), 1);
        assert_eq!(instance.hostfwd[0].to_qemu(), "tcp:127.0.0.1:9000-:90");
        assert!(!instance.isolate);
        assert_eq!(instance.execute, Some("echo hello".to_string()));
    }

    #[test]
    fn test_defaults_apply_without_a_template() {
        let instance = build_instance_from(&["create", "web"], None);

        assert_eq!(instance.user.as_str(), "host-user");
        assert_eq!(instance.cpus, 2);
        assert_eq!(instance.mem.get_bytes(), GIB);
        assert_eq!(instance.disk_capacity, DEFAULT_DISK_SIZE);
        assert!(instance.hostfwd.is_empty());
        assert_eq!(instance.execute, None);
        assert!(!instance.isolate);
    }

    #[test]
    fn test_resolve_image_prefers_the_command_line_and_needs_a_source() {
        let template = build_template();
        let resolve = |args: &[&str], template: Option<&Template>| {
            CreateCommand::try_parse_from(args)
                .unwrap()
                .resolve_image(template)
        };

        let image = resolve(&["create", "web", "-i", "ubuntu:noble"], Some(&template)).unwrap();
        assert_eq!(image.get_name(), "noble");

        let image = resolve(&["create", "web"], Some(&template)).unwrap();
        assert_eq!(image.get_name(), "trixie");

        assert!(matches!(
            resolve(&["create", "web"], None),
            Err(Error::MissingImage)
        ));
    }
}
