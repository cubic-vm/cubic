use crate::commands::{self, Verbosity};
use crate::error::Error;
use crate::instance::InstanceDao;
use crate::util::Terminal;
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

impl InstanceConsoleCommand {
    pub fn run(&self, instance_dao: &InstanceDao) -> Result<(), Error> {
        commands::start(
            instance_dao,
            &None,
            Verbosity::Quiet,
            true,
            &vec![self.instance.to_string()],
        )?;

        let console_path = instance_dao.env.get_console_file(&self.instance);
        while !Path::new(&console_path).exists() {
            thread::sleep(Duration::new(1, 0));
        }

        if let Ok(mut term) = Terminal::open(&console_path) {
            term.wait();
        } else {
            println!("Cannot open shell");
        }

        Ok(())
    }
}
