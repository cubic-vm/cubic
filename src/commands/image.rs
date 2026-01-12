use crate::commands;
use crate::commands::Command;
use crate::env::Environment;
use crate::error::Error;
use crate::fs::FS;
use crate::image::{Image, ImageFactory, ImageFetcher, ImageName, ImageStore};
use crate::instance::InstanceStore;
use crate::view::{Console, SpinnerView};
use clap::Subcommand;

pub fn fetch_image_list(env: &Environment) -> Vec<Image> {
    let mut spinner = SpinnerView::new("Fetching image list".to_string());
    let images: Vec<Image> = ImageFactory::new(env).create_images().unwrap_or_default();
    spinner.stop();
    images
}

pub fn fetch_image_info(env: &Environment, image: &ImageName) -> Result<Image, Error> {
    let mut spinner = SpinnerView::new(format!(
        "Looking up image {}:{}",
        image.get_vendor(),
        image.get_name()
    ));
    let image = ImageFactory::new(env).get_image(image);
    spinner.stop();
    image
}

pub fn fetch_image(
    env: &Environment,
    image_store: &dyn ImageStore,
    image: &Image,
) -> Result<(), Error> {
    if !image_store.exists(image) {
        FS::new().create_dir(&env.get_image_dir())?;
        ImageFetcher::new().fetch(
            image,
            &format!("{}/{}", env.get_image_dir(), image.to_file_name()),
        )?;
    }
    Ok(())
}

#[derive(Subcommand)]
pub enum ImageCommands {
    #[clap(alias = "list")]
    Ls(commands::ListImageCommand),

    /// Fetch an image
    Fetch {
        /// Name of the virtual machine image
        image: ImageName,
    },

    Info(commands::ShowImageCommand),
    Prune(commands::PruneCommand),
}

impl Command for ImageCommands {
    fn run(
        &self,
        console: &mut dyn Console,
        env: &Environment,
        image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<(), Error> {
        match self {
            ImageCommands::Ls(cmd) => cmd.run(console, env, image_store, instance_store),
            ImageCommands::Info(cmd) => cmd.run(console, env, image_store, instance_store),
            ImageCommands::Fetch { image } => {
                let image = &fetch_image_info(env, image)?;
                fetch_image(env, image_store, image)
            }

            ImageCommands::Prune(cmd) => cmd.run(console, env, image_store, instance_store),
        }
    }
}
