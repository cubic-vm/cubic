use crate::commands::{self, Command, image::fetch_image_info};
use crate::error::Result;
use crate::image::ImageName;
use crate::view::{Console, MapView};
use clap::Parser;

/// Show VM images
#[derive(Parser)]
pub struct ShowImageCommand {
    /// Name of the virtual machine image
    pub name: ImageName,
}

impl Command for ShowImageCommand {
    fn run(&self, console: &mut dyn Console, context: &commands::Context) -> Result<()> {
        let image = fetch_image_info(context.get_env(), &self.name)?;
        let mut view = MapView::new();
        view.add("Name", &image.get_image_names());
        view.add("Image URL", &image.image_url);
        view.add("Checksum URL", &image.checksum_url);
        view.print(console);
        Ok(())
    }
}
