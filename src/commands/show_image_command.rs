use crate::commands::{self, Command, image::fetch_image_info};
use crate::error::Result;
use crate::image::ImageStore;
use crate::models::{DataSize, ImageName};
use crate::util;
use crate::view::{Console, MapView};
use clap::Parser;

/// Show VM images
#[derive(Parser)]
pub struct ShowImageCommand {
    /// Name of the virtual machine image
    pub name: ImageName,

    #[clap(flatten)]
    pub all: commands::AllInfoArg,
}

impl Command for ShowImageCommand {
    fn run(&self, console: &mut Console<'_>, context: &commands::Context) -> Result<()> {
        let env = context.get_env();
        let image = fetch_image_info(console, context.get_system(), env, &self.name)?;

        let mut view = MapView::new();
        view.add("Name", &image.get_image_names());
        view.add("Architecture", &image.arch.to_string());
        if let Some(size) = image.size {
            view.add("Size", &DataSize::new(size as usize).to_size());
        }
        view.add(
            "Cached",
            util::to_yes_no(ImageStore::new().exists(context.get_system(), env, &image)),
        );

        if self.all.value {
            view.add("Checksum", &image.hash_alg.to_string());
            view.add(
                "Image File",
                &format!("{}/{}", env.get_image_dir(), image.to_file_name()),
            );
            view.add("Image URL", &image.image_url);
            view.add("Checksum URL", &image.checksum_url);
        }

        view.print(console);
        Ok(())
    }
}
