use crate::error::Error;
use crate::machine::{MachineDao, MachineState};
use crate::ssh_cmd::{get_ssh_private_key_names, Ssh};

use std::io::Write;
use std::os::unix::process::CommandExt;

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
    verbose: bool,
    ssh_args: &Option<String>,
    cmd: &Option<String>,
) -> Result<(), Error> {
    let instance = get_instance_name(target)?;
    let machine = machine_dao.load(&instance)?;
    let user = get_user_name(target)?.unwrap_or(machine.user.to_string());
    let ssh_port = machine.ssh_port;
    let mut stdout = std::io::stdout();

    if !machine_dao.is_running(&machine) {
        machine_dao.start(&machine, &None, false, verbose)?;
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

    Ssh::new()
        .set_private_keys(get_ssh_private_key_names()?)
        .set_port(Some(ssh_port))
        .set_xforward(xforward)
        .set_args(ssh_args.clone().unwrap_or_default())
        .set_user(user)
        .set_cmd(cmd.clone())
        .set_verbose(verbose)
        .connect()
        .exec();

    Ok(())
}
