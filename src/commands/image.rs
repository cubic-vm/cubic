use crate::commands;
use crate::env::Environment;
use crate::error::Error;
use crate::fs::FS;
use crate::image::{Image, ImageDao, ImageFactory, ImageFetcher, ImageStore};
use crate::view::{Console, MapView, SpinnerView};
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
        image: String,
    },

    /// Show image information
    Info {
        /// Name of the virtual machine image
        name: String,
    },

    Prune(commands::PruneCommand),
}

impl ImageCommands {
    pub fn dispatch(&self, console: &mut dyn Console, image_dao: &ImageDao) -> Result<(), Error> {
        match self {
            ImageCommands::Ls(cmd) => cmd.run(console, &image_dao.env),
            ImageCommands::Info { name } => {
                fetch_image_list(&image_dao.env);
                let image = image_dao.get(name)?;
                let mut view = MapView::new();
                view.add("Vendor", &image.vendor);
                view.add("Codename", &image.codename);
                view.add("Version", &image.version);
                view.add("URL", &image.url);
                view.print(console);
                Ok(())
            }
            ImageCommands::Fetch { image } => {
                fetch_image_list(&image_dao.env);
                let image = &image_dao.get(image)?;

                if !image_dao.exists(image) {
                    FS::new().create_dir(&image_dao.env.get_image_dir())?;
                    ImageFetcher::new().fetch(
                        image,
                        &format!("{}/{}", image_dao.env.get_image_dir(), image.to_file_name()),
                    )?;
                }

                Ok(())
            }

            ImageCommands::Prune(cmd) => cmd.run(image_dao),
        }
    }
}
