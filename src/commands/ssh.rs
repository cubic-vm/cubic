use crate::commands::{self, Verbosity};
use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore};
use crate::ssh_cmd::{get_ssh_private_key_names, Ssh};

use std::env;
use std::os::unix::process::CommandExt;
use std::thread::sleep;
use std::time::Duration;

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
    instance_dao: &InstanceDao,
    target: &str,
    xforward: bool,
    verbosity: Verbosity,
    ssh_args: &Option<String>,
    cmd: &Option<String>,
) -> Result<(), Error> {
    let name = get_instance_name(target)?;
    let instance = instance_dao.load(&name)?;
    let user = get_user_name(target)?.unwrap_or(instance.user.to_string());
    let ssh_port = instance.ssh_port;

    if !instance_dao.is_running(&instance) {
        commands::start(instance_dao, &None, verbosity, &vec![name.to_string()])?;
        sleep(Duration::from_millis(3000));
    }

    Err(Error::Io(
        Ssh::new()
            .set_known_hosts_file(
                env::var("HOME")
                    .map(|dir| format!("{dir}/.ssh/known_hosts"))
                    .ok(),
            )
            .set_private_keys(get_ssh_private_key_names()?)
            .set_port(Some(ssh_port))
            .set_xforward(xforward)
            .set_args(ssh_args.clone().unwrap_or_default())
            .set_user(user.clone())
            .set_cmd(cmd.clone())
            .set_verbose(verbosity.is_verbose())
            .connect()
            .exec(),
    ))
}
