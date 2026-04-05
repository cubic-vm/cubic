use crate::commands::{Command, Context, fetch_image_list};
use crate::error::Result;
use crate::image::get_default_arch;
use crate::model::DataSize;
use crate::view::{Alignment, Console, TableView};
use clap::Parser;

/// List VM images
///
/// Examples:
///
///   $ cubic images
///   Name                       Arch         Size   Cached
///   archlinux:latest           amd64   518.7 MiB       no
///   debian:{12, bookworm}      amd64   424.2 MiB       no
///   debian:{11, bullseye}      amd64   343.8 MiB       no
///   debian:{10, buster}        amd64   301.7 MiB       no
///   debian:{13, trixie}        amd64   412.0 MiB      yes
///   fedora:41                  amd64   468.9 MiB       no
///   fedora:42                  amd64   507.6 MiB       no
///   fedora:43                  amd64   556.3 MiB       no
///   opensuse:15.5              amd64   643.1 MiB       no
///   [...]
///   opensuse:15.6              amd64   682.7 MiB       no
///   [...]
///   rockylinux:10              amd64   548.8 MiB       no
///   rockylinux:8               amd64     1.9 GiB       no
///   rockylinux:9               amd64   618.8 MiB       no
///   [...]
///   ubuntu:{24.04, noble}      amd64   250.4 MiB      yes
///   [...]
///
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct ListImageCommand {
    /// Show all images
    #[clap(short, long, action, global = true)]
    all: bool,
}

impl Command for ListImageCommand {
    fn run(&self, console: &mut dyn Console, context: &Context) -> Result<()> {
        let images = fetch_image_list(context.get_env());

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
                    if context.get_image_store().exists(&image) {
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
