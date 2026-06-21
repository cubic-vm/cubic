use crate::commands::{self, Command, image::fetch_image_info};
use crate::error::Result;
use crate::models::{DataSize, ImageName};
use crate::util;
use crate::view::{Console, MapView};
use clap::Parser;

/// Show VM images
#[derive(Parser)]
pub struct ShowImageCommand {
    /// Name of the virtual machine image
    pub name: ImageName,

    /// Show all available information
    #[arg(short = 'a', long = "all")]
    pub all: bool,
}

impl Command for ShowImageCommand {
    fn run(&self, console: &mut dyn Console, context: &commands::Context) -> Result<()> {
        let env = context.get_env();
        let image = fetch_image_info(env, &self.name)?;

        let size = image
            .size
            .map(|size| DataSize::new(size as usize).to_size())
            .unwrap_or("n/a".to_string());

        let mut view = MapView::new();
        view.add("Name", &image.get_image_names());
        view.add("Architecture", &image.arch.to_string());
        view.add("Size", &size);
        view.add(
            "Cached",
            util::to_yes_no(context.get_image_store().exists(&image)),
        );

        if self.all {
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
