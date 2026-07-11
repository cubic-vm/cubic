use crate::commands::{self, Command};
use crate::error::Result;
use crate::fs::FS;
use crate::models::DataSize;
use crate::view::{ConfirmDialog, Console};
use clap::Parser;

/// Clear caches
///
/// This commands removes cached VM images files.
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct PruneCommand {
    #[clap(flatten)]
    yes: commands::YesArg,
}

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

        console.print("The following items will be deleted:");
        console.print("  - VM image list cache");
        console.print("  - Downloaded VM images");

        // Print size of files to be deleted
        console.print(&format!("\nTotal size: {total}\n"));

        if self.yes.value
            || ConfirmDialog::new("Are you sure you want to continue?").confirm(console)
        {
            // Delete files
            fs.remove_file(&env.get_image_cache_file()).ok();
            fs.remove_dir(&env.get_image_dir()).ok();

            // Print size of deleted files
            console.print(&format!("Successfully freed {total} of disk space."));
        }

        Ok(())
    }
}
