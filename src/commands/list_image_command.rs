use crate::commands::{Command, fetch_image_list};
use crate::env::Environment;
use crate::error::Error;
use crate::image::{ImageStore, get_default_arch};
use crate::instance::InstanceStore;
use crate::model::DataSize;
use crate::view::{Alignment, Console, TableView};
use clap::Parser;

/// List all supported virtual machine images
#[derive(Parser)]
pub struct ListImageCommand {
    /// Show all images
    #[clap(short, long, action, global = true)]
    all: bool,
}

impl Command for ListImageCommand {
    fn run(
        &self,
        console: &mut dyn Console,
        env: &Environment,
        image_store: &dyn ImageStore,
        _instance_store: &dyn InstanceStore,
    ) -> Result<(), Error> {
        let images = fetch_image_list(env);

        let mut view = TableView::new();
        view.add_row()
            .add("Name", Alignment::Left)
            .add("Arch", Alignment::Left)
            .add("Size", Alignment::Right)
            .add("Cached", Alignment::Right);

        for image in images {
            if !self.all && image.arch != get_default_arch() {
                continue;
            }

            let size = image
                .size
                .map(|size| DataSize::new(size as usize).to_size())
                .unwrap_or_default();

            view.add_row()
                .add(&image.get_image_names(), Alignment::Left)
                .add(&image.arch.to_string(), Alignment::Left)
                .add(&size, Alignment::Right)
                .add(
                    if image_store.exists(&image) {
                        "yes"
                    } else {
                        "no"
                    },
                    Alignment::Right,
                );
        }
        view.print(console);
        Ok(())
    }
}
