use crate::error::Error;
use crate::image::ImageStore;
use clap::Parser;

/// Clear cache and free space
#[derive(Parser)]
pub struct PruneCommand;

impl PruneCommand {
    pub fn run(&self, image_store: &dyn ImageStore) -> Result<(), Error> {
        image_store.prune()
    }
}
