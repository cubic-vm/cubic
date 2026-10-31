use crate::actions::LoadInstanceAction;
use crate::commands::{self, Command};
use crate::error::{Error, Result};
use crate::models::{TargetInstancePath, TargetPath};
use crate::ssh::SshClient;
use clap::Parser;

fn resolve_target_path(
    context: &commands::Context,
    target_path: &TargetPath,
) -> Result<TargetInstancePath> {
    if let Some(target) = target_path.get_target() {
        let instance = LoadInstanceAction::new().run(context, target.get_instance().as_str())?;
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

fn check_target_is_running(context: &commands::Context, target: &TargetPath) -> Result<()> {
    if let Some(target) = target.get_target() {
        let instance_store = context.get_instance_store();
        let instance = LoadInstanceAction::new().run(context, target.get_instance().as_str())?;
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
///   Upload the current directory to the VM instance 'trixie':
///   $ cubic scp . trixie:~/
///
///   Download a directory from the VM instance 'trixie' to host:
///   $ cubic scp trixie:~/Downloads .
///
///   Copy a file from the VM instance 'trixie' to 'noble':
///   $ cubic scp trixie:~/cubic.tar.gz noble:~/
///
#[derive(Parser)]
#[clap(alias = "cp", verbatim_doc_comment)]
pub struct ScpCommand {
    /// Source of the data to copy
    from: TargetPath,
    /// Target of the data to copy
    to: TargetPath,
}

impl Command for ScpCommand {
    async fn run(&self, context: &commands::Context) -> Result<u8> {
        check_target_is_running(context, &self.from)?;
        check_target_is_running(context, &self.to)?;

        let env = context.get_env();
        let from = resolve_target_path(context, &self.from)?;
        let to = resolve_target_path(context, &self.to)?;

        context
            .get_console()
            .debug(&format!("Copying '{}' to '{}'", self.from, self.to));

        let mut ssh = SshClient::new(context);
        ssh.set_private_keys(env.get_home_ssh_private_key_paths(context.get_system()));
        let source = self.from.to_string();
        ssh.copy(&from, &to, &source).await?;
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::ContextBuilder;
    use crate::models::Instance;
    use crate::platform::SystemMock;
    use std::str::FromStr;

    #[test]
    fn test_check_local_path_needs_no_instance() {
        let context = ContextBuilder::new(SystemMock::new()).build();
        let path = TargetPath::from_str("/home/cubic/file").unwrap();

        assert!(check_target_is_running(&context, &path).is_ok());
    }

    #[test]
    fn test_check_rejects_stopped_instance() {
        let mut builder = ContextBuilder::new(SystemMock::new());
        builder.get_store().add_instances(vec![Instance {
            name: "test".to_string(),
            ..Instance::default()
        }]);
        let context = builder.build();
        let path = TargetPath::from_str("test:~/file").unwrap();

        assert!(matches!(
            check_target_is_running(&context, &path),
            Err(Error::InstanceNotRunning(ref name)) if name == "test"
        ));
    }
}
