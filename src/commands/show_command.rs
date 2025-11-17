use crate::commands::{self, Command};
use crate::env::Environment;
use crate::error::Error;
use crate::image::ImageStore;
use crate::instance::InstanceStore;
use crate::model::InstanceImageName;
use crate::view::Console;
use clap::Parser;

/// Show virtual machine image or instance information
#[derive(Parser)]
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
    ) -> Result<(), Error> {
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
