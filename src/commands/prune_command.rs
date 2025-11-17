use crate::commands::Command;
use crate::env::Environment;
use crate::error::Error;
use crate::image::ImageStore;
use crate::instance::InstanceStore;
use crate::view::Console;
use clap::Parser;

/// Clear cache and free space
#[derive(Parser)]
pub struct PruneCommand;

impl Command for PruneCommand {
    fn run(
        &self,
        _console: &mut dyn Console,
        _env: &Environment,
        image_store: &dyn ImageStore,
        _instance_store: &dyn InstanceStore,
    ) -> Result<(), Error> {
        image_store.prune()
    }
}
