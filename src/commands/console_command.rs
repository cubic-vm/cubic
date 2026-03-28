use crate::commands::{self, Command, Iso9660};
use crate::env::Environment;
use crate::error::Result;
use crate::image::ImageStore;
use crate::instance::InstanceStore;
#[cfg(not(windows))]
use crate::util::Terminal;
use crate::view::Console;
use clap::Parser;
use std::path::Path;
use std::thread;
use std::time::Duration;

/// Open VM instance console
///
/// Examples:
///
///   Connect to the console of 'my-instance'
///   $ cubic console my-instance
///   Default credentials: cubic / cubic
///   Press CTRL+W to exit the console.
///
///   [...]
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct ConsoleCommand {
    /// Name of the virtual machine instance
    instance: String,
    /// Switch for Rust and system ISO9600 implementation
    #[clap(hide = true)]
    #[arg(value_enum, long, default_value_t = Iso9660::System)]
    pub iso9660: Iso9660,
}

impl Command for ConsoleCommand {
    fn run(
        &self,
        console: &mut dyn Console,
        env: &Environment,
        image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<()> {
        commands::StartCommand {
            qemu_args: None,
            wait: false,
            instances: vec![self.instance.to_string()],
            iso9660: self.iso9660.clone(),
        }
        .run(console, env, image_store, instance_store)?;

        console.info("Default credentials: cubic / cubic");
        console.info("Press CTRL+W to exit the console.");
        let console_path = env.get_console_file(&self.instance);
        while !Path::new(&console_path).exists() {
            thread::sleep(Duration::new(1, 0));
        }

        console.raw_mode();
        #[cfg(not(windows))]
        if let Ok(mut term) = Terminal::open(&console_path) {
            term.wait();
        } else {
            console.error("Cannot open shell");
        }
        console.reset();
        Ok(())
    }
}
