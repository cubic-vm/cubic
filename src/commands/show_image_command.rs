use crate::commands::image::fetch_image_list;
use crate::env::Environment;
use crate::error::Error;
use crate::image::{ImageName, ImageStore};
use crate::view::{Console, MapView};
use clap::Parser;

/// Show image information
#[derive(Parser)]
pub struct ShowImageCommand {
    /// Name of the virtual machine image
    name: ImageName,
}

impl ShowImageCommand {
    pub fn run(
        &self,
        console: &mut dyn Console,
        env: &Environment,
        image_store: &dyn ImageStore,
    ) -> Result<(), Error> {
        fetch_image_list(env);
        let image = image_store.get(&self.name)?;
        let mut view = MapView::new();
        view.add("Vendor", &image.vendor);
        view.add("Codename", &image.codename);
        view.add("Version", &image.version);
        view.add("URL", &image.url);
        view.print(console);
        Ok(())
    }
}
