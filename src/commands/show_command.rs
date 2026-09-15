use crate::commands::{self, Command};
use crate::error::Result;
use crate::models::{ImageName, InstanceName};
use crate::util::Either;
use crate::view::Console;
use clap::Parser;
use std::sync::Arc;

/// Show VM images and instances
///
/// Use this command to inspect VM instance configuration and VM image details.
///
/// Examples:
///
///   Show information of a VM instance
///   $ cubic show trixie
///   Running:    yes
///   Arch:       amd64
///   vCPUs:      6
///   Memory:     16 G
///   Disk Used:  5325 M
///   Disk Total: 100 G
///   User:       cubic
///   Isolated:   no
///   Forward:    127.0.0.1:4000:4000/tcp
///
///   Show all information, adding the process id, ports, file locations and the SSH command
///   $ cubic show --all trixie
///   ... (fields above, then)
///   PID:          12345
///   SSH Port:     54315
///   Monitor Port: 54316
///   Console Port: 54317
///   Disk Image:   ~/.local/share/cubic/machines/trixie/machine.img
///   Config:       ~/.local/share/cubic/machines/trixie/instance.toml
///   SSH Key:      ~/.local/share/cubic/machines/trixie/ssh_client_key
///   SSH Host Key: SHA256:ZkAslGjFiUHdGf/WUL8rQvkib4PTvQatUV0OUQSncCA
///   SSH:          ssh -i .../trixie/ssh_client_key -p 54315 cubic@localhost
///
///   Show information of a VM image
///   A plain name is an instance, so an image needs a name or a tag
///   $ cubic show ubuntu:latest
///   Name:   ubuntu:26.04
///   Tags:   resolute, stable, latest
///   Arch:   amd64
///   Size:   408 M
///   Cached: yes
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
    ///
    /// A plain name is an instance. An image needs a name or a tag,
    /// for example ubuntu:latest.
    name: Either<InstanceName, ImageName>,

    #[clap(flatten)]
    all: commands::AllInfoArg,
}

impl Command for ShowCommand {
    async fn run(&self, console: &Arc<Console>, context: &commands::Context) -> Result<u8> {
        match &self.name {
            Either::Left(instance) => {
                commands::ShowInstanceCommand {
                    instance: instance.clone().into(),
                    all: self.all.value.into(),
                }
                .run(console, context)
                .await
            }
            Either::Right(name) => {
                commands::ShowImageCommand {
                    name: name.clone(),
                    all: self.all.value.into(),
                }
                .run(console, context)
                .await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::instance::InstanceStoreMock;
    use crate::models::{Environment, Instance, UserName};
    use crate::platform::{System, SystemMock};
    use std::str::FromStr;
    use std::sync::Arc;

    fn build_context(instances: Vec<Instance>) -> commands::Context {
        let env = Environment::new(
            UserName::from_str("cubic").unwrap(),
            String::new(),
            String::new(),
        );
        commands::Context::new(
            Arc::new(SystemMock::new()),
            env,
            Box::new(InstanceStoreMock::new(instances)),
        )
    }

    #[tokio::test]
    async fn test_show_routes_plain_name_to_instance_view() {
        let system = SystemMock::new();
        let system = Arc::new(system);
        let console = &Console::new(Arc::clone(&system) as Arc<dyn System>);
        let context = build_context(vec![Instance {
            name: "test".to_string(),
            ..Instance::default()
        }]);

        ShowCommand {
            name: "test".parse().unwrap(),
            all: false.into(),
        }
        .run(console, &context)
        .await
        .unwrap();

        assert!(system.get_output().starts_with("Running:"));
    }

    #[tokio::test]
    async fn test_show_rejects_unknown_instance() {
        let system = SystemMock::new();
        let system = Arc::new(system);
        let console = &Console::new(Arc::clone(&system) as Arc<dyn System>);
        let context = build_context(Vec::new());

        let result = ShowCommand {
            name: "missing".parse().unwrap(),
            all: false.into(),
        }
        .run(console, &context)
        .await;

        assert!(matches!(
            result,
            Err(Error::UnknownInstance(ref name)) if name == "missing"
        ));
    }
}
