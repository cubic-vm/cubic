use crate::commands::{AllImagesArg, Command, Context};
use crate::error::Result;
use crate::image::{ImageFactory, ImageStore};
use crate::models::Arch;
use crate::view::{Alignment, TableView};
use clap::Parser;

/// List VM images
///
/// Examples:
///
///   $ cubic images
///   Name                 OS              Tags                              Arch    Cached
///   Arch Linux           archlinux       rolling, stable, latest           amd64       no
///   Debian 12            debian          12, bookworm                      amd64       no
///   Debian 13            debian          13, trixie, stable, latest        amd64      yes
///   Fedora 43            fedora          43                                amd64       no
///   Fedora 44            fedora          44, stable, latest                amd64       no
///   [...]
///   Ubuntu 24.04         ubuntu          24.04, noble                      amd64       no
///   Ubuntu 26.04         ubuntu          26.04, resolute, stable, latest   amd64      yes
///   [...]
///
///   Use the OS to get the stable release, or add a tag to pick another one,
///   so ubuntu:26.04, ubuntu:resolute and ubuntu:latest are the same image.
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
            .add("OS", Alignment::Left)
            .add("Tags", Alignment::Left)
            .add("Arch", Alignment::Left)
            .add("Cached", Alignment::Right);

        for image in images
            .into_iter()
            .filter(|image| image.arch == arch && (self.all.value || !image.eol))
        {
            view.add_row()
                .add(&ImageFactory::get_display_name(&image), Alignment::Left)
                .add(&image.distro, Alignment::Left)
                .add(&image.get_tags(), Alignment::Left)
                .add(&image.arch.to_string(), Alignment::Left)
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
