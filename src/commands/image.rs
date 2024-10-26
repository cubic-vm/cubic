use crate::error::Error;
use crate::image::ImageDao;
use crate::util;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum ImageCommands {
    /// List images
    List {
        #[clap(short, long, default_value_t = false)]
        all: bool,
    },

    /// Fetch an image
    Fetch { image: String },

    /// Delete images
    Del { images: Vec<String> },
}

impl ImageCommands {
    pub fn dispatch(&self, image_dao: &ImageDao) -> Result<(), Error> {
        match self {
            ImageCommands::List { all } => {
                println!(
                    "{:10}  {:>7}  {:10}  {: >5}  {: >9}",
                    "Vendor", "Version", "Name", "Arch", "Size"
                );
                for image in image_dao.get_images() {
                    if !(*all || image_dao.exists(image)) {
                        continue;
                    }

                    let size = image_dao
                        .get_disk_size(image)
                        .map(util::bytes_to_human_readable)
                        .unwrap_or_default();
                    println!(
                        "{:10}  {:>7}  {:10}  {: >5}  {: >9}",
                        image.vendor, image.version, image.codename, "amd64", size
                    )
                }

                Ok(())
            }

            ImageCommands::Fetch { image } => image_dao.fetch(&image_dao.get(image)?),

            ImageCommands::Del { images } => {
                for name in images {
                    let image = image_dao.get(name)?;

                    if !image_dao.exists(&image) {
                        return Result::Err(Error::UnknownImage(image.to_id()));
                    }

                    if util::confirm(&format!(
                        "Do you really want delete the image '{name}'? [y/n]: "
                    )) {
                        image_dao.delete(&image)?;
                        println!("Deleted image {name}");
                    }
                }

                Ok(())
            }
        }
    }
}
