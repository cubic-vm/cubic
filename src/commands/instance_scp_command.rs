use crate::commands::Command;
use crate::env::Environment;
use crate::error::Error;
use crate::image::ImageStore;
use crate::instance::{InstanceStore, TargetPath};
use crate::ssh_cmd::{Openssh, Russh, Ssh, get_ssh_private_key_names};
use crate::view::Console;
use clap::Parser;
use std::env;

/// Copy a file from or to a virtual machine instance with SCP
#[derive(Parser)]
pub struct InstanceScpCommand {
    /// Source of the data to copy
    from: TargetPath,
    /// Target of the data to copy
    to: TargetPath,
    /// Pass additional SCP arguments
    #[clap(long)]
    scp_args: Option<String>,
    #[clap(long, conflicts_with = "russh", default_value_t = false, hide = true)]
    pub openssh: bool,
    #[clap(long, conflicts_with = "openssh", default_value_t = false, hide = true)]
    pub russh: bool,
}

impl Command for InstanceScpCommand {
    fn run(
        &self,
        console: &mut dyn Console,
        _env: &Environment,
        _image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<(), Error> {
        let from = &self.from.to_scp(instance_store)?;
        let to = &self.to.to_scp(instance_store)?;
        let root_dir = env::var("SNAP").unwrap_or_default();

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
        ssh.set_args(self.scp_args.clone().unwrap_or_default());
        ssh.copy(console, &root_dir, from, to);
        Ok(())
    }
}
