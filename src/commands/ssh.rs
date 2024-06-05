use crate::error::Error;
use crate::machine::{MachineDao, USER};
use crate::util;
use std::net::TcpStream;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::{thread::sleep, time::Duration};

pub fn ssh(machine_dao: &MachineDao, name: &str, cmd: &Option<String>) -> Result<(), Error> {
    util::check_ssh_key();

    let mut user = USER;
    let mut instance = name;

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
        machine_dao.start(&machine)?;
    }

    // wait for SSH connection
    for i in 1..30 {
        if TcpStream::connect(format!("127.0.0.1:{ssh_port}")).is_ok() {
            if i == 30 {
                return Result::Err(Error::ConnectionTimeout(name.to_string()));
            }
            break;
        } else {
            sleep(Duration::from_millis(1000));
        }
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
