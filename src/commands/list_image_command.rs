use crate::commands::{AllImagesArg, Command, Context};
use crate::error::Result;
use crate::image::{ImageFactory, ImageStore};
use crate::models::{Arch, DataSize};
use crate::view::{Alignment, TableView};
use clap::Parser;

/// List VM images
///
/// Examples:
///
///   $ cubic images
///   Name                Tags                       Arch     Size   Cached
///   archlinux:rolling   stable, latest             amd64   531 M       no
///   debian:12           bookworm                   amd64   429 M       no
///   debian:13           trixie, stable, latest     amd64   414 M      yes
///   fedora:43                                      amd64   556 M       no
///   fedora:44           stable, latest             amd64   557 M       no
///   [...]
///   ubuntu:24.04        noble                      amd64   252 M       no
///   ubuntu:26.04        resolute, stable, latest   amd64   408 M      yes
///   [...]
///
///   Use the name of a row or swap its version for one of its tags, so
///   ubuntu:26.04, ubuntu:resolute and ubuntu:latest are the same image.
///   Every distribution carries two extra tags. The tag latest is the newest
///   release and the tag stable is the newest long term release, which is the
///   last LTS for Ubuntu and the current stable for Debian. A rolling release
///   such as archlinux:rolling has no version and uses the same image for both.
///
///   Releases without security updates are hidden. Use --all to show them.
///   Use --arch to list the images of another architecture.
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct ListImageCommand {
    #[clap(flatten)]
    all: AllImagesArg,

    /// Show images of this architecture, defaults to the host architecture
    #[clap(long = "arch", value_name = "amd64|arm64")]
    arch: Option<Arch>,
}

impl Command for ListImageCommand {
    async fn run(&self, context: &Context) -> Result<u8> {
        let images = ImageFactory::get_all_images();
        let arch = self.arch.unwrap_or_else(Arch::get_host);

        let mut view = TableView::new();
        view.add_row()
            .add("Name", Alignment::Left)
            .add("Tags", Alignment::Left)
            .add("Arch", Alignment::Left)
            .add("Size", Alignment::Right)
            .add("Cached", Alignment::Right);

        for image in images
            .into_iter()
            .filter(|image| image.arch == arch && (self.all.value || !image.eol))
        {
            let size = image
                .size
                .map(|size| DataSize::new(size as usize).to_size())
                .unwrap_or_default();

            view.add_row()
                .add(&image.get_image_name(), Alignment::Left)
                .add(&image.get_tags(), Alignment::Left)
                .add(&image.arch.to_string(), Alignment::Left)
                .add(&size, Alignment::Right)
                .add(
                    if ImageStore::new().exists(context.get_system(), context.get_env(), &image) {
                        "yes"
                    } else {
                        "no"
                    },
                    Alignment::Right,
                );
        }
        view.print(context.get_console());
        Ok(0)
    }
}
