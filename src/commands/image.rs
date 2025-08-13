use crate::commands;
use crate::env::Environment;
use crate::error::Error;
use crate::fs::FS;
use crate::image::{Image, ImageDao, ImageFactory, ImageFetcher, ImageStore};
use crate::util;
use crate::view::SpinnerView;
use crate::view::{Alignment, TableView};
use crate::view::{MapView, Stdio};
use clap::Subcommand;

fn fetch_image_list(env: &Environment) -> Vec<Image> {
    let mut spinner = SpinnerView::new("Fetching image list");
    let images: Vec<Image> = ImageFactory::new(env).create_images().unwrap_or_default();
    spinner.stop();
    images
}

#[derive(Subcommand)]
pub enum ImageCommands {
    /// List images
    #[clap(alias = "list")]
    Ls {
        /// List all images
        #[clap(short, long, hide = true)]
        all: bool,
    },

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
    pub fn dispatch(&self, image_dao: &ImageDao) -> Result<(), Error> {
        let console = &mut Stdio::new();

        match self {
            ImageCommands::Ls { .. } => {
                let images = fetch_image_list(&image_dao.env);

                let mut view = TableView::new();
                view.add_row()
                    .add("Name", Alignment::Left)
                    .add("Arch", Alignment::Left)
                    .add("Size", Alignment::Right);

                for image in images {
                    let size = image
                        .size
                        .map(util::bytes_to_human_readable)
                        .unwrap_or_default();

                    view.add_row()
                        .add(
                            &format!("{}:{}", image.vendor, image.version),
                            Alignment::Left,
                        )
                        .add(&image.arch.to_string(), Alignment::Left)
                        .add(&size, Alignment::Right);

                    if image.version != image.codename {
                        view.add_row()
                            .add(
                                &format!("{}:{}", image.vendor, image.codename),
                                Alignment::Left,
                            )
                            .add(&image.arch.to_string(), Alignment::Left)
                            .add(&size, Alignment::Right);
                    }
                }
                view.print(console);
                Ok(())
            }

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
