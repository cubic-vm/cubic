use crate::commands::Verbosity;
use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore};
use crate::ssh_cmd::{get_ssh_private_key_names, Scp};
use std::env;
use std::os::unix::process::CommandExt;

fn get_scp_address(instance_dao: &InstanceDao, location: &str) -> Result<String, Error> {
    Ok(if location.contains(':') {
        let mut location_token = location.split(':');
        let name = location_token.next().unwrap();
        let path = location_token.next().unwrap();
        let instance = instance_dao.load(name)?;
        let port = instance.ssh_port;
        let user = instance.user;
        format!("scp://{user}@127.0.0.1:{port}/{path}")
    } else {
        location.to_string()
    })
}

pub fn scp(
    instance_dao: &InstanceDao,
    from: &str,
    to: &str,
    verbosity: Verbosity,
    scp_args: &Option<String>,
) -> Result<(), Error> {
    let from = &get_scp_address(instance_dao, from)?;
    let to = &get_scp_address(instance_dao, to)?;

    Err(Error::Io(
        Scp::new()
            .set_root_dir(env::var("SNAP").unwrap_or_default().as_str())
            .set_verbose(verbosity.is_verbose())
            .set_known_hosts_file(
                env::var("HOME")
                    .map(|dir| format!("{dir}/.ssh/known_hosts"))
                    .ok(),
            )
            .set_private_keys(get_ssh_private_key_names()?)
            .set_args(scp_args.as_ref().unwrap_or(&String::new()))
            .copy(from, to)
            .exec(),
    ))
}
