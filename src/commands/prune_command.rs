use crate::commands::Command;
use crate::env::Environment;
use crate::error::Result;
use crate::image::ImageStore;
use crate::instance::InstanceStore;
use crate::view::Console;
use clap::Parser;

/// Clear caches
///
/// This commands removes cached VM images files.
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct PruneCommand;

impl Command for PruneCommand {
    fn run(
        &self,
        _console: &mut dyn Console,
        _env: &Environment,
        image_store: &dyn ImageStore,
        _instance_store: &dyn InstanceStore,
    ) -> Result<()> {
        image_store.prune()
    }
}
