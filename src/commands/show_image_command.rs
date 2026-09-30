use crate::commands::{self, Command};
use crate::error::Result;
use crate::image::{ImageFactory, ImageStore};
use crate::models::ImageName;
use crate::util;
use crate::view::MapView;
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
    async fn run(&self, context: &commands::Context) -> Result<u8> {
        let env = context.get_env();
        let image = ImageFactory::find_image(&self.name)?;

        let mut view = MapView::new();
        view.add("Name", &ImageFactory::get_display_name(&image));
        view.add("OS", &image.distro);
        view.add("Tags", &image.get_tags());
        view.add("Arch", &image.arch.to_string());
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
            view.add("Image URL", &image.get_image_url());
            view.add("Checksum URL", &image.get_checksum_url());
        }

        view.print(context.get_console());
        Ok(0)
    }
}
