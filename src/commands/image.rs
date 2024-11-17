use crate::error::Error;
use crate::image::ImageDao;
use crate::util;
use crate::view::{Alignment, TableView};
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
                let mut view = TableView::new();
                view.add_row()
                    .add("Vendor", Alignment::Left)
                    .add("Version", Alignment::Right)
                    .add("Name", Alignment::Left)
                    .add("Arch", Alignment::Left)
                    .add("Size", Alignment::Right);

                for image in image_dao.get_images() {
                    if !(*all || image_dao.exists(image)) {
                        continue;
                    }

                    let size = image_dao
                        .get_disk_size(image)
                        .map(util::bytes_to_human_readable)
                        .unwrap_or_default();

                    view.add_row()
                        .add(&image.vendor, Alignment::Left)
                        .add(&image.version, Alignment::Right)
                        .add(&image.codename, Alignment::Left)
                        .add("amd64", Alignment::Left)
                        .add(&size, Alignment::Right);
                }
                view.print();
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
