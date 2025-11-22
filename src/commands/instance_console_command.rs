use crate::commands::{self, Command};
use crate::env::Environment;
use crate::error::Error;
use crate::image::ImageStore;
use crate::instance::InstanceStore;
use crate::util::Terminal;
use crate::view::Console;
use clap::Parser;
use std::path::Path;
use std::thread;
use std::time::Duration;

/// Open the console of an virtual machine instance
#[derive(Parser)]
pub struct InstanceConsoleCommand {
    /// Name of the virtual machine instance
    instance: String,
}

impl Command for InstanceConsoleCommand {
    fn run(
        &self,
        console: &mut dyn Console,
        env: &Environment,
        image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<(), Error> {
        commands::InstanceStartCommand {
            qemu_args: None,
            wait: false,
            instances: vec![self.instance.to_string()],
        }
        .run(console, env, image_store, instance_store)?;

        println!("Press CTRL+W to exit the console.");
        let console_path = env.get_console_file(&self.instance);
        while !Path::new(&console_path).exists() {
            thread::sleep(Duration::new(1, 0));
        }

        console.raw_mode();
        if let Ok(mut term) = Terminal::open(&console_path) {
            term.wait();
        } else {
            println!("Cannot open shell");
        }
        console.reset();
        Ok(())
    }
}
