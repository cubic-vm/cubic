use crate::commands::{self, Verbosity};
use crate::error::Error;
use crate::instance::{InstanceDao, CONSOLE_COUNT};
use crate::util::Terminal;

use std::path::Path;
use std::thread;
use std::time::Duration;

pub fn sh(
    instance_dao: &InstanceDao,
    console: bool,
    verbosity: Verbosity,
    name: &str,
) -> Result<(), Error> {
    let instance = instance_dao.load(name)?;

    if !instance_dao.is_running(&instance) {
        commands::start(instance_dao, &None, verbosity, &vec![name.to_string()])?;
    }

    if console {
        let console_path = format!("{}/{}/console", instance_dao.cache_dir, name);
        while !Path::new(&console_path).exists() {
            thread::sleep(Duration::new(1, 0));
        }

        if let Ok(mut term) = Terminal::open(&console_path) {
            term.wait();
        } else {
            println!("Cannot open shell");
        }
    } else {
        for i in 1..CONSOLE_COUNT {
            let console_path = format!("{}/{}/console{i}", instance_dao.cache_dir, name);
            if let Ok(mut term) = Terminal::open(&console_path) {
                term.wait();
                return Ok(());
            }
        }

        println!("All shells are occupied");
    }

    Ok(())
}
