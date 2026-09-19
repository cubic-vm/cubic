use crate::commands::{self, Command};
use crate::error::Result;
use clap::Parser;

/// Restart VM instances
///
/// Examples:
///
///   Restart the VM instance 'my-instance':
///   $ cubic restart my-instance
///
///   Restart multiple VM instances:
///   $ cubic restart trixie noble
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct RestartCommand {
    #[clap(flatten)]
    pub accel: commands::AccelArg,
    #[clap(flatten)]
    instances: commands::InstancesArg,
}

impl Command for RestartCommand {
    async fn run(&self, context: &commands::Context) -> Result<u8> {
        commands::StopCommand {
            all: false.into(),
            wait: true,
            kill: false,
            instances: self.instances.value.clone().into(),
        }
        .run(context)
        .await?;
        commands::StartCommand {
            qemu_args: None,
            accel: self.accel,
            wait: true,
            yes: commands::YesArg { value: false },
            instances: self.instances.value.clone().into(),
        }
        .run(context)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reject_path_traversal() {
        assert!(RestartCommand::try_parse_from(["restart", "../../etc"]).is_err());
    }
}
