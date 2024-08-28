use crate::error::Error;
use crate::machine::{MachineDao, MachineState};
use crate::util;
use std::io::Write;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, Instant};

fn get_instance_name(target: &str) -> Result<String, Error> {
    if target.contains('@') {
        target
            .split('@')
            .nth(1)
            .map(|instance| instance.to_string())
            .ok_or(Error::InvalidSshTarget(target.to_string()))
    } else {
        Ok(target.to_string())
    }
}

fn get_user_name(target: &str) -> Result<Option<String>, Error> {
    if target.contains('@') {
        target
            .split('@')
            .next()
            .map(|instance| Some(instance.to_string()))
            .ok_or(Error::InvalidSshTarget(target.to_string()))
    } else {
        Ok(None)
    }
}

pub fn ssh(
    machine_dao: &MachineDao,
    target: &str,
    xforward: bool,
    ssh_args: &Option<String>,
    cmd: &Option<String>,
) -> Result<(), Error> {
    util::check_ssh_key();

    let instance = get_instance_name(target)?;
    let machine = machine_dao.load(&instance)?;
    let user = get_user_name(target)?.unwrap_or(machine.user.to_string());
    let ssh_port = machine.ssh_port;
    let mut stdout = std::io::stdout();

    if !machine_dao.is_running(&machine) {
        machine_dao.start(&machine, &None, false)?;
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
        .arg(ssh_port.to_string());

    if xforward {
        command.arg("-X");
    }

    if let Some(ssh_args) = ssh_args {
        for arg in ssh_args.split(' ') {
            command.arg(arg);
        }
    }

    command
        .arg(format!("{user}@127.0.0.1"))
        .arg(cmd.as_deref().unwrap_or(""))
        .exec();

    Result::Ok(())
}
