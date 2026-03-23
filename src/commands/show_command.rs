use crate::commands::{self, Command};
use crate::env::Environment;
use crate::error::Result;
use crate::image::ImageStore;
use crate::instance::InstanceStore;
use crate::model::InstanceImageName;
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
///   Arch:       amd64
///   CPUs:       6
///   Memory:     16.0 GiB
///   Disk Used:  5.2 GiB
///   Disk Total: 100.0 GiB
///   User:       cubic
///   Isolated:   no
///   SSH Port:   54315
///   SSH:        ssh -p 54315 cubic@localhost
///   Forward:    127.0.0.1:4000:4000/tcp
///
///   Show information of a VM image
///   $ cubic show ubuntu:noble
///   Name:         ubuntu:{24.04, noble}
///   Image URL:    https://cloud-images.ubuntu.com/minimal/releases/noble/release/...
///   Checksum URL: https://cloud-images.ubuntu.com/minimal/releases/noble/release/...
///
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct ShowCommand {
    /// Name of the virtual machine image or instance
    name: InstanceImageName,
}

impl Command for ShowCommand {
    fn run(
        &self,
        console: &mut dyn Console,
        env: &Environment,
        image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<()> {
        match &self.name {
            InstanceImageName::Image(name) => commands::ShowImageCommand { name: name.clone() }
                .run(console, env, image_store, instance_store),
            InstanceImageName::Instance(instance) => commands::InstanceShowCommand {
                instance: instance.clone(),
            }
            .run(console, env, image_store, instance_store),
        }
    }
}
