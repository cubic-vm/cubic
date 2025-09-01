use crate::commands::fetch_image_list;
use crate::env::Environment;
use crate::error::Error;
use crate::model::DataSize;
use crate::view::{Alignment, Console, TableView};
use clap::Parser;

/// List all supported virtual machine images
#[derive(Parser)]
pub struct ListImageCommand;

impl ListImageCommand {
    pub fn run(&self, console: &mut dyn Console, env: &Environment) -> Result<(), Error> {
        let images = fetch_image_list(env);

        let mut view = TableView::new();
        view.add_row()
            .add("Name", Alignment::Left)
            .add("Arch", Alignment::Left)
            .add("Size", Alignment::Right);

        for image in images {
            let size = image
                .size
                .map(|size| DataSize::new(size as usize).to_size())
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
}
