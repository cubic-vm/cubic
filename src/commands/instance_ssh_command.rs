use crate::commands::{self, Verbosity};
use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore, Target};
use crate::ssh_cmd::{get_ssh_private_key_names, Ssh};
use crate::view::SpinnerView;
use clap::Parser;
use std::env;
use std::thread;
use std::time::{Duration, Instant};

/// Connect to a virtual machine instance with SSH
#[derive(Parser)]
pub struct InstanceSshCommand {
    /// Target instance (format: [username@]instance, e.g. 'cubic@mymachine' or 'mymachine')
    pub target: Target,
    /// Forward X over SSH
    #[clap(short = 'X', default_value_t = false)]
    pub xforward: bool,
    /// Pass additional SSH arguments
    #[clap(long)]
    pub ssh_args: Option<String>,
    /// Execute a command in the virtual machine
    pub cmd: Option<String>,
}

impl InstanceSshCommand {
    pub fn run(&self, instance_dao: &InstanceDao, verbosity: Verbosity) -> Result<(), Error> {
        let name = self.target.get_instance();
        let instance = instance_dao.load(name.as_str())?;
        let user = self
            .target
            .get_user()
            .map(|user| user.to_string())
            .unwrap_or(instance.user.to_string());
        let ssh_port = instance.ssh_port;

        commands::InstanceStartCommand {
            qemu_args: None,
            wait: true,
            instances: vec![name.to_string()],
        }
        .run(instance_dao, verbosity)?;

        let mut ssh = None;
        let mut start_time = Instant::now();

        if !verbosity.is_quiet() {
            println!("Default login user: cubic / password: cubic");
        }

        loop {
            if ssh.is_none() {
                ssh = Some(
                    Ssh::new()
                        .set_known_hosts_file(
                            env::var("HOME")
                                .map(|dir| format!("{dir}/.ssh/known_hosts"))
                                .ok(),
                        )
                        .set_private_keys(get_ssh_private_key_names()?)
                        .set_port(Some(ssh_port))
                        .set_xforward(self.xforward)
                        .set_args(self.ssh_args.clone().unwrap_or_default())
                        .set_user(user.clone())
                        .set_cmd(self.cmd.clone())
                        .set_verbose(verbosity.is_verbose())
                        .connect()
                        .spawn()?,
                );
                start_time = Instant::now();
            }

            if let Ok(Some(exit)) = ssh.as_mut().unwrap().try_wait() {
                if exit.success() || self.cmd.is_some() || start_time.elapsed().as_secs() > 5 {
                    break;
                }
                let spinner = (!verbosity.is_quiet()).then(|| SpinnerView::new("Try to connect"));
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
}
