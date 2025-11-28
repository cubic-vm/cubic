use crate::commands::{Command, image::fetch_image_list};
use crate::env::Environment;
use crate::error::Error;
use crate::image::{ImageName, ImageStore};
use crate::instance::InstanceStore;
use crate::view::{Console, MapView};
use clap::Parser;

/// Show image information
#[derive(Parser)]
pub struct ShowImageCommand {
    /// Name of the virtual machine image
    pub name: ImageName,
}

impl Command for ShowImageCommand {
    fn run(
        &self,
        console: &mut dyn Console,
        env: &Environment,
        image_store: &dyn ImageStore,
        _instance_store: &dyn InstanceStore,
    ) -> Result<(), Error> {
        fetch_image_list(env);
        let image = image_store.get(&self.name)?;
        let mut view = MapView::new();
        view.add("Name", &image.get_image_names());
        view.add("URL", &image.url);
        view.print(console);
        Ok(())
    }
}
