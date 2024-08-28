use crate::error::Error;
use crate::machine::MachineDao;
use std::process::Command;

pub fn scp(machine_dao: &MachineDao, from: &str, to: &str) -> Result<(), Error> {
    if from.contains(':') {
        let mut from_token = from.split(':');
        let name = from_token.next().unwrap();
        let path = from_token.next().unwrap();

        let machine = machine_dao.load(name)?;
        let user = machine.user;
        let ssh_port = machine.ssh_port;

        Command::new("scp")
            .arg("-r")
            .arg("-P")
            .arg(ssh_port.to_string())
            .arg(format!("{user}@127.0.0.1:{path}"))
            .arg(to)
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    } else {
        let mut to_token = to.split(':');
        let name = to_token.next().unwrap();
        let path = to_token.next().unwrap();

        let machine = machine_dao.load(name)?;
        let user = machine.user;
        let ssh_port = machine.ssh_port;

        Command::new("scp")
            .arg("-r")
            .arg("-P")
            .arg(ssh_port.to_string())
            .arg(from)
            .arg(format!("{user}@127.0.0.1:{path}"))
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }

    Result::Ok(())
}
