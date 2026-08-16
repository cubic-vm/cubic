use crate::actions::LoadInstanceAction;
use crate::commands::{self, Command};
use crate::error::{Error, Result};
use crate::models::{TargetInstancePath, TargetPath};
use crate::ssh::SshClient;
use crate::util;
use crate::view::Console;
use clap::Parser;

fn resolve_target_path(
    context: &commands::Context,
    console: &mut Console<'_>,
    target_path: &TargetPath,
) -> Result<TargetInstancePath> {
    if let Some(target) = target_path.get_target() {
        let instance =
            LoadInstanceAction::new().run(context, console, target.get_instance().as_str())?;
        Ok(TargetInstancePath {
            user: target.get_user().map(|user| user.to_string()),
            instance: Some(instance),
            path: target_path.path.clone(),
        })
    } else {
        Ok(TargetInstancePath {
            user: None,
            instance: None,
            path: target_path.path.clone(),
        })
    }
}

fn check_target_is_running(
    context: &commands::Context,
    console: &mut Console<'_>,
    target: &TargetPath,
) -> Result<()> {
    if let Some(target) = target.get_target() {
        let instance_store = context.get_instance_store();
        let instance =
            LoadInstanceAction::new().run(context, console, target.get_instance().as_str())?;
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
    fn run(&self, console: &mut Console<'_>, context: &commands::Context) -> Result<()> {
        check_target_is_running(context, console, &self.from)?;
        check_target_is_running(context, console, &self.to)?;

        let env = context.get_env();
        let from = resolve_target_path(context, console, &self.from)?;
        let to = resolve_target_path(context, console, &self.to)?;
        let from_key = from
            .instance
            .as_ref()
            .map(|instance| env.get_ssh_private_key_file(&instance.name));
        let to_key = to
            .instance
            .as_ref()
            .map(|instance| env.get_ssh_private_key_file(&instance.name));

        console.debug(&format!("Copying '{}' to '{}'", self.from, self.to));

        let mut ssh = SshClient::new(context);
        ssh.set_private_keys(env.get_home_ssh_private_key_paths(context.get_system()));
        util::AsyncCaller::new().call(ssh.copy(
            console,
            &from,
            from_key.as_deref(),
            &to,
            to_key.as_deref(),
        ))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::InstanceStoreMock;
    use crate::models::{Environment, Instance, UserName};
    use crate::platform::SystemMock;
    use std::rc::Rc;
    use std::str::FromStr;

    fn build_context(instance_store: InstanceStoreMock) -> commands::Context {
        let env = Environment::new(
            UserName::from_str("cubic").unwrap(),
            String::new(),
            String::new(),
        );
        commands::Context::new(Rc::new(SystemMock::new()), env, Box::new(instance_store))
    }

    #[test]
    fn test_check_local_path_needs_no_instance() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let context = build_context(InstanceStoreMock::new(Vec::new()));
        let path = TargetPath::from_str("/home/cubic/file").unwrap();

        assert!(check_target_is_running(&context, console, &path).is_ok());
    }

    #[test]
    fn test_check_rejects_unknown_instance() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let context = build_context(InstanceStoreMock::new(Vec::new()));
        let path = TargetPath::from_str("missing:~/file").unwrap();

        assert!(matches!(
            check_target_is_running(&context, console, &path),
            Err(Error::UnknownInstance(ref name)) if name == "missing"
        ));
    }

    #[test]
    fn test_check_rejects_stopped_instance() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let context = build_context(InstanceStoreMock::new(vec![Instance {
            name: "test".to_string(),
            ..Instance::default()
        }]));
        let path = TargetPath::from_str("test:~/file").unwrap();

        assert!(matches!(
            check_target_is_running(&context, console, &path),
            Err(Error::InstanceNotRunning(ref name)) if name == "test"
        ));
    }
}
