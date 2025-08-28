use crate::commands;
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

impl ShowCommand {
    pub fn run(
        self,
        console: &mut dyn Console,
        env: &Environment,
        instance_store: &dyn InstanceStore,
        image_store: &dyn ImageStore,
    ) -> Result<(), Error> {
        match self.name {
            InstanceImageName::Image(name) => {
                commands::ShowImageCommand { name }.run(console, env, image_store)
            }
            InstanceImageName::Instance(instance) => {
                commands::InstanceShowCommand { instance }.run(console, instance_store)
            }
        }
    }
}
