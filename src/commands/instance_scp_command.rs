use crate::commands::Command;
use crate::env::Environment;
use crate::error::Error;
use crate::image::ImageStore;
use crate::instance::{InstanceStore, TargetPath};
use crate::ssh_cmd::{Openssh, Russh, Ssh, get_ssh_private_key_names};
use crate::view::Console;
use clap::Parser;
use std::env;

fn check_target_is_running(
    instance_store: &dyn InstanceStore,
    target: &TargetPath,
) -> Result<(), Error> {
    if let Some(target) = target.get_target() {
        let instance = instance_store.load(target.get_instance().as_str())?;
        if !instance_store.is_running(&instance) {
            return Err(Error::InstanceNotRunning(instance.name.clone()));
        }
    }
    Ok(())
}

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
        check_target_is_running(instance_store, &self.from)?;
        check_target_is_running(instance_store, &self.to)?;

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
        ssh.copy(
            console,
            &root_dir,
            &self.from.to_target_instance_path(instance_store)?,
            &self.to.to_target_instance_path(instance_store)?,
        );
        Ok(())
    }
}
