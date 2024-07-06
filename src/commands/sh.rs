use crate::error::Error;
use crate::machine::{MachineDao, MachineState, CONSOLE_COUNT};
use crate::util::Terminal;
use std::path::Path;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

pub fn sh(machine_dao: &MachineDao, console: bool, name: &str) -> Result<(), Error> {
    let machine = machine_dao.load(name)?;

    if !machine_dao.is_running(&machine) {
        machine_dao.start(&machine, false)?;
    }

    if console {
        let console_path = format!("{}/{}/console", machine_dao.cache_dir, name);
        while !Path::new(&console_path).exists() {
            thread::sleep(Duration::new(1, 0));
        }

        if let Ok(mut term) = Terminal::open(&console_path) {
            term.run();
        } else {
            println!("Cannot open shell");
        }
    } else {
        while machine_dao.get_state(&machine) == MachineState::Stopped {
            sleep(Duration::new(1, 0));
        }

        for i in 1..CONSOLE_COUNT {
            let console_path = format!("{}/{}/console{i}", machine_dao.cache_dir, name);
            if let Ok(mut term) = Terminal::open(&console_path) {
                term.run();
                return Ok(());
            }
        }

        println!("All shells are occupied");
    }

    Ok(())
}
