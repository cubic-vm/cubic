use crate::commands::{self, Verbosity};
use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore, Target};
use crate::ssh_cmd::{Openssh, Russh, Ssh, get_ssh_private_key_names};
use crate::view::{Console, SpinnerView};
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
    /// Select the ssh client library (openssh or russh are supported)
    #[clap(long, conflicts_with = "russh", default_value_t = false, hide = true)]
    pub openssh: bool,
    #[clap(long, conflicts_with = "openssh", default_value_t = false, hide = true)]
    pub russh: bool,
    /// Execute a command in the virtual machine
    pub cmd: Option<String>,
}

impl InstanceSshCommand {
    pub fn run(
        &self,
        console: &mut dyn Console,
        instance_dao: &InstanceDao,
        verbosity: Verbosity,
    ) -> Result<(), Error> {
        let name = self.target.get_instance();

        commands::InstanceStartCommand {
            qemu_args: None,
            wait: true,
            instances: vec![name.to_string()],
        }
        .run(instance_dao, verbosity)?;

        let instance = instance_dao.load(name.as_str())?;
        let user = self
            .target
            .get_user()
            .map(|user| user.to_string())
            .unwrap_or(instance.user.to_string());
        let ssh_port = instance.ssh_port;

        let mut ssh: Box<dyn Ssh> = if !self.russh {
            Box::new(Openssh::new())
        } else {
            Box::new(Russh::new())
        };
        ssh.set_known_hosts_file(
            env::var("HOME")
                .map(|dir| format!("{dir}/.ssh/known_hosts"))
                .ok(),
        );
        ssh.set_private_keys(get_ssh_private_key_names()?);
        ssh.set_port(Some(ssh_port));
        ssh.set_xforward(self.xforward);
        ssh.set_args(self.ssh_args.clone().unwrap_or_default());
        ssh.set_user(user.clone());
        ssh.set_cmd(self.cmd.clone());
        console.debug(&ssh.get_command());

        console.info("Default login user: cubic / password: cubic");

        loop {
            let start_time = Instant::now();
            if ssh.connect() {
                // exit on success
                break;
            }

            if self.ssh_args.is_some() || start_time.elapsed().as_secs() > 5 {
                // exit if cli command or time expired
                break;
            }

            let spinner = (!verbosity.is_quiet()).then(|| SpinnerView::new("Try to connect"));
            thread::sleep(Duration::from_secs(5));
            if let Some(mut s) = spinner {
                s.stop()
            }
        }

        Ok(())
    }
}
