use crate::commands::{self, Command};
use crate::error::{Error, Result};
use crate::fs::FS;
use crate::instance::{InstanceStore, resolve_target_path};
use crate::models::TargetPath;
use crate::ssh_cmd::Russh;
use crate::view::Console;
use clap::Parser;
use std::env;

fn check_target_is_running(instance_store: &dyn InstanceStore, target: &TargetPath) -> Result<()> {
    if let Some(target) = target.get_target() {
        let instance = instance_store.load(target.get_instance().as_str())?;
        if !instance_store.is_running(&instance) {
            return Err(Error::InstanceNotRunning(instance.name.clone()));
        }
    }
    Ok(())
}

/// Copy data between host and VM instances
///
/// Data can be copied from host to VM instance, from VM instance to host, and
/// between VM instances:
/// $ cubic scp <path/to/host/file> <instance>:<path/to/guest/file>
/// $ cubic scp <instance>:<path/to/guest/file> <path/to/host/file>
/// $ cubic scp <instance>:<path/to/guest/file> <instance>:<path/to/guest/file>
///
/// Examples:
///
///   Upload a file from host to the VM instance 'trixie':
///   $ cubic scp ./cubic.tar.gz trixie:~/
///
///   Download a directory from the VM instance 'trixie' to host:
///   $ cubic scp trixie:~/Downloads .
///
///   Copy a file from the VM instance 'trixie' to 'noble':
///   $ cubic scp trixie:~/cubic.tar.gz noble:~/
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct ScpCommand {
    /// Source of the data to copy
    from: TargetPath,
    /// Target of the data to copy
    to: TargetPath,
}

impl Command for ScpCommand {
    fn run(&self, console: &mut dyn Console, context: &commands::Context) -> Result<()> {
        let instance_store = context.get_instance_store();
        check_target_is_running(instance_store, &self.from)?;
        check_target_is_running(instance_store, &self.to)?;

        let env = context.get_env();
        let root_dir = env::var("SNAP").unwrap_or_default();

        let from = resolve_target_path(&self.from, instance_store)?;
        let to = resolve_target_path(&self.to, instance_store)?;
        let from_key = from
            .instance
            .as_ref()
            .map(|instance| env.get_ssh_private_key_file(&instance.name));
        let to_key = to
            .instance
            .as_ref()
            .map(|instance| env.get_ssh_private_key_file(&instance.name));

        let mut ssh = Russh::new();
        ssh.set_private_keys(env.get_home_ssh_private_key_paths(&FS::new()));
        ssh.copy(
            console,
            &root_dir,
            &from,
            from_key.as_deref(),
            &to,
            to_key.as_deref(),
        )?;
        Ok(())
    }
}
