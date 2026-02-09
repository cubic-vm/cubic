use crate::commands::Command;
use crate::env::Environment;
use crate::error::Error;
use crate::fs::FS;
use crate::image::ImageStore;
use crate::instance::{InstanceStore, TargetPath};
use crate::ssh_cmd::Russh;
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
}

impl Command for InstanceScpCommand {
    fn run(
        &self,
        console: &mut dyn Console,
        env: &Environment,
        _image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<(), Error> {
        check_target_is_running(instance_store, &self.from)?;
        check_target_is_running(instance_store, &self.to)?;

        let root_dir = env::var("SNAP").unwrap_or_default();
        let mut ssh = Russh::new();

        let pubkeys = env.get_ssh_private_key_paths(
            &FS::new(),
            [&self.from, &self.to]
                .iter()
                .filter_map(|target_path| {
                    target_path
                        .get_target()
                        .map(|target| target.get_instance().to_string())
                })
                .collect(),
        );

        ssh.set_private_keys(pubkeys);
        ssh.copy(
            console,
            &root_dir,
            &self.from.to_target_instance_path(instance_store)?,
            &self.to.to_target_instance_path(instance_store)?,
        );
        Ok(())
    }
}
