use crate::commands::{self, Command};
use crate::error::Result;
use crate::view::Console;
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
    instances: commands::InstancesArg,
}

impl Command for RestartCommand {
    fn run(&self, console: &mut Console<'_>, context: &commands::Context) -> Result<()> {
        commands::StopCommand {
            all: false,
            wait: true,
            kill: false,
            instances: self.instances.value.clone().into(),
        }
        .run(console, context)?;
        commands::StartCommand {
            qemu_args: None,
            wait: true,
            yes: commands::YesArg { value: false },
            instances: self.instances.value.clone().into(),
        }
        .run(console, context)
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
