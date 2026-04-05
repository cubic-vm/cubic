use crate::commands::{self, Command};
use crate::error::Result;
use crate::fs::FS;
use crate::model::DataSize;
use crate::util;
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
    fn run(&self, console: &mut dyn Console, context: &commands::Context) -> Result<()> {
        let env = context.get_env();

        // Calculate size
        let fs = FS::new();
        let paths = [env.get_image_cache_file(), env.get_image_dir()];
        let total = DataSize::new(
            paths
                .iter()
                .fold(0, |total, path| total + fs.get_size(path)) as usize,
        )
        .to_size();

        console.info("The following items will be deleted:");
        console.info("  - VM image list cache");
        console.info("  - Downloaded VM images");

        // Print size of files to be deleted
        console.info(&format!("\nTotal size: {total}\n"));

        if util::confirm("Are you sure you want to continue? [y/N]") {
            // Delete files
            fs.remove_file(&env.get_image_cache_file()).ok();
            fs.remove_dir(&env.get_image_dir()).ok();

            // Print size of deleted files
            console.info(&format!("Successfully freed {total} of disk space."));
        }

        Ok(())
    }
}
