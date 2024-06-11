use crate::error::Error;
use crate::machine::{MachineDao, MachineState, USER};
use crate::util;
use std::io::Write;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, Instant};

pub fn ssh(machine_dao: &MachineDao, name: &str, cmd: &Option<String>) -> Result<(), Error> {
    util::check_ssh_key();

    let mut user = USER;
    let mut instance = name;
    let mut stdout = std::io::stdout();

    if name.contains('@') {
        let mut tokens = name.split('@');
        user = tokens
            .next()
            .ok_or(Error::InvalidSshTarget(name.to_string()))?;
        instance = tokens
            .next()
            .ok_or(Error::InvalidSshTarget(name.to_string()))?;
    }

    let machine = machine_dao.load(instance)?;
    let ssh_port = machine.ssh_port;

    if !machine_dao.is_running(&machine) {
        machine_dao.start(&machine, false)?;
    }

    if machine_dao.get_state(&machine) != MachineState::Running {
        // wait for SSH connection
        let start = Instant::now();
        let mut running: Option<Instant> = Option::None;
        while running
            .map(|running| running.elapsed() < Duration::new(5, 0))
            .unwrap_or(true)
        {
            sleep(Duration::from_millis(1000));
            print!(
                "\rWaiting for machine to start... {:04.0?}",
                start.elapsed()
            );
            stdout.flush().ok();

            if running.is_none() && machine_dao.get_state(&machine) == MachineState::Running {
                running = Some(Instant::now());
            }
        }
        println!();
    }

    let mut command = Command::new("ssh");

    for key in util::get_ssh_key_names()? {
        command.arg("-i").arg(key);
    }

    command
        .arg("-o")
        .arg("StrictHostKeyChecking=no")
        .arg("-p")
        .arg(ssh_port.to_string())
        .arg(format!("{user}@127.0.0.1"))
        .arg(cmd.as_deref().unwrap_or(""))
        .exec();

    Result::Ok(())
}
