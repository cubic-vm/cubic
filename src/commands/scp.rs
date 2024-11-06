use crate::error::Error;
use crate::machine::MachineDao;
use crate::ssh_cmd::{get_ssh_private_key_names, Scp};
use std::env;
use std::os::unix::process::CommandExt;

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

    Scp::new()
        .set_root_dir(env::var("SNAP").unwrap_or_default().as_str())
        .set_verbose(verbose)
        .set_private_keys(get_ssh_private_key_names()?)
        .set_port(Some(ssh_port))
        .set_args(scp_args.as_ref().unwrap_or(&String::new()))
        .copy(from, to)
        .exec();

    Ok(())
}
