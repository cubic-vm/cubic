use crate::error::Error;
use crate::machine::MachineDao;
use crate::util;
use std::process::Command;

fn resolve_name(machine_dao: &MachineDao, location: &str) -> Result<String, Error> {
    Ok(if location.contains(':') {
        let mut location_token = location.split(':');
        let name = location_token.next().unwrap();
        let path = location_token.next().unwrap();
        let machine = machine_dao.load(name)?;
        let user = machine.user;
        format!("{user}@127.0.0.1:{path}")
    } else {
        location.to_string()
    })
}

fn resolve_port(machine_dao: &MachineDao, location: &str) -> Result<Option<u16>, Error> {
    Ok(if location.contains(':') {
        let mut location_token = location.split(':');
        let name = location_token.next().unwrap();
        let machine = machine_dao.load(name)?;
        Some(machine.ssh_port)
    } else {
        None
    })
}

pub fn scp(
    machine_dao: &MachineDao,
    from: &str,
    to: &str,
    verbose: bool,
    scp_args: &Option<String>,
) -> Result<(), Error> {
    let ssh_port = resolve_port(machine_dao, from)?
        .or(resolve_port(machine_dao, to)?)
        .unwrap();
    let from = &resolve_name(machine_dao, from)?;
    let to = &resolve_name(machine_dao, to)?;

    let mut command = Command::new("scp");
    for key in util::get_ssh_key_names()? {
        command.arg("-i").arg(key);
    }

    command
        .arg("-o")
        .arg("StrictHostKeyChecking=no")
        .arg("-r")
        .arg("-P")
        .arg(ssh_port.to_string());

    if let Ok(snap_root) = std::env::var("SNAP") {
        command.arg(format!("-S{snap_root}/usr/bin/ssh"));
    }

    if let Some(scp_args) = scp_args {
        for arg in scp_args.split(' ') {
            command.arg(arg);
        }
    }

    command.arg(from).arg(to);

    if verbose {
        util::print_command(&command);
    }

    command.spawn().unwrap().wait().unwrap();

    Result::Ok(())
}
