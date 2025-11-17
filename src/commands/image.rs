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
    let mut spinner = SpinnerView::new("Fetching image list");
    let images: Vec<Image> = ImageFactory::new(env).create_images().unwrap_or_default();
    spinner.stop();
    images
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
                fetch_image_list(env);
                let image = &image_store.get(image)?;

                if !image_store.exists(image) {
                    FS::new().create_dir(&env.get_image_dir())?;
                    ImageFetcher::new().fetch(
                        image,
                        &format!("{}/{}", env.get_image_dir(), image.to_file_name()),
                    )?;
                }

                Ok(())
            }

            ImageCommands::Prune(cmd) => cmd.run(console, env, image_store, instance_store),
        }
    }
}
