use crate::commands::{self, Verbosity};
use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore};
use crate::ssh_cmd::{get_ssh_private_key_names, PortChecker, Ssh};
use crate::view::SpinnerView;
use std::env;
use std::thread;
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

    if instance_dao.is_running(&instance) {
        let spinner = (!verbosity.is_quiet()).then(|| SpinnerView::new("Connecting to instance"));

        // Waiting for SSH server to be available
        while !PortChecker::new(ssh_port).try_connect() {
            thread::sleep(Duration::from_secs(1));
        }

        if let Some(mut s) = spinner {
            s.stop()
        }
    } else {
        commands::start(instance_dao, &None, verbosity, &vec![name.to_string()])?;
    }

    let mut ssh = None;
    let mut start_time = Instant::now();
    loop {
        if ssh.is_none() {
            if !verbosity.is_quiet() {
                println!("Default login user: cubic / password: cubic");
            }

            ssh = Some(
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
                    .spawn()
                    .unwrap(),
            );
            start_time = Instant::now();
        }

        if let Ok(Some(exit)) = ssh.as_mut().unwrap().try_wait() {
            if exit.success() || cmd.is_some() || start_time.elapsed().as_secs() > 5 {
                break;
            }
            let spinner = (!verbosity.is_quiet()).then(|| SpinnerView::new("Connection retry"));
            thread::sleep(Duration::from_secs(5));
            if let Some(mut s) = spinner {
                s.stop()
            }
            ssh = None;
        } else {
            thread::sleep(Duration::from_secs(1));
        }
    }

    Ok(())
}
