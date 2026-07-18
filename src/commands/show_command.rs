use crate::commands::{self, Command};
use crate::error::Result;
use crate::models::InstanceImageName;
use crate::view::Console;
use clap::Parser;

/// Show VM images and instances
///
/// Use this command to inspect VM instance configuration and VM image details.
///
/// Examples:
///
///   Show information of a VM instance
///   $ cubic show trixie
///   Status:       running
///   PID:          12345
///   Arch:         amd64
///   CPUs:         6
///   Memory:       16.0 GiB
///   Disk Used:    5.2 GiB
///   Disk Total:   100.0 GiB
///   User:         cubic
///   Isolated:     no
///   SSH Port:     54315
///   Monitor Port: 54316
///   Console Port: 54317
///
///   Show all information, adding file locations, the SSH command and forwards
///   $ cubic show --all trixie
///   ... (fields above, then)
///   Disk Image:   ~/.local/share/cubic/machines/trixie/machine.img
///   Config:       ~/.local/share/cubic/machines/trixie/instance.toml
///   SSH Key:      ~/.local/share/cubic/machines/trixie/ssh_client_key
///   SSH:          ssh -i .../trixie/ssh_client_key -p 54315 cubic@localhost
///   Forward:      127.0.0.1:4000:4000/tcp
///
///   Show information of a VM image
///   $ cubic show ubuntu:noble
///   Name:         ubuntu:{24.04, noble}
///   Architecture: amd64
///   Size:         512.0 MiB
///   Cached:       yes
///
///   Show all image information, adding checksum, file path and URLs
///   $ cubic show --all ubuntu:noble
///   ... (fields above, then)
///   Checksum:     sha256
///   Image File:   ~/.cache/cubic/images/ubuntu_noble_amd64
///   Image URL:    https://cloud-images.ubuntu.com/minimal/releases/noble/...
///   Checksum URL: https://cloud-images.ubuntu.com/minimal/releases/noble/...
///
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct ShowCommand {
    /// Name of the virtual machine image or instance
    name: InstanceImageName,

    /// Show all available information
    #[arg(short = 'a', long = "all")]
    all: bool,
}

impl Command for ShowCommand {
    fn run(&self, console: &mut Console<'_>, context: &commands::Context) -> Result<()> {
        match &self.name {
            InstanceImageName::Image(name) => commands::ShowImageCommand {
                name: name.clone(),
                all: self.all,
            }
            .run(console, context),
            InstanceImageName::Instance(instance) => commands::ShowInstanceCommand {
                instance: instance.clone().into(),
                all: self.all,
            }
            .run(console, context),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::image::ImageStoreMock;
    use crate::instance::InstanceStoreMock;
    use crate::models::{Environment, Instance, UserName};
    use crate::platform::SystemMock;
    use std::rc::Rc;
    use std::str::FromStr;

    fn build_context(instances: Vec<Instance>) -> commands::Context {
        let env = Environment::new(
            UserName::from_str("cubic").unwrap(),
            String::new(),
            String::new(),
            String::new(),
        );
        commands::Context::new(
            Rc::new(SystemMock::new()),
            env,
            Box::new(ImageStoreMock::default()),
            Box::new(InstanceStoreMock::new(instances)),
        )
    }

    #[test]
    fn test_show_routes_plain_name_to_instance_view() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let context = build_context(vec![Instance {
            name: "test".to_string(),
            ..Instance::default()
        }]);

        ShowCommand {
            name: InstanceImageName::from_str("test").unwrap(),
            all: false,
        }
        .run(console, &context)
        .unwrap();

        assert!(system.get_output().starts_with("Running:"));
    }

    #[test]
    fn test_show_rejects_unknown_instance() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let context = build_context(Vec::new());

        let result = ShowCommand {
            name: InstanceImageName::from_str("missing").unwrap(),
            all: false,
        }
        .run(console, &context);

        assert!(matches!(
            result,
            Err(Error::UnknownInstance(ref name)) if name == "missing"
        ));
    }
}
