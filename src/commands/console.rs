use crate::commands::{self, Verbosity};
use crate::error::Error;
use crate::instance::InstanceDao;
use crate::util::Terminal;

use std::path::Path;
use std::str;
use std::thread;
use std::time::Duration;

pub fn console(instance_dao: &InstanceDao, name: &str) -> Result<(), Error> {
    commands::start(
        instance_dao,
        &None,
        Verbosity::Quiet,
        true,
        &vec![name.to_string()],
    )?;

    let console_path = instance_dao.env.get_console_file(name);
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
