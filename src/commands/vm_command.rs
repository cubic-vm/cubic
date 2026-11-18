use crate::actions::{LoadInstanceAction, StartInstanceAction};
use crate::commands::{self, Command};
use crate::error::Result;
use crate::vm::VmServer;
use clap::Parser;
use std::time::Duration;

const ACCEPT_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Parser)]
#[clap(hide = true)]
pub struct VmCommand {
    #[clap(long)]
    pub qemu_args: Option<String>,
    #[clap(flatten)]
    pub accel: commands::AccelArg,
    #[clap(flatten)]
    pub instance: commands::InstanceArg,
}

impl Command for VmCommand {
    async fn run(&self, context: &commands::Context) -> Result<u8> {
        let instance = LoadInstanceAction::new().run(context, self.instance.value.as_str())?;
        let server = VmServer::bind(context.get_system(), context.get_env(), &instance)?;
        let mut action = StartInstanceAction::new(&instance);
        let mut run = std::pin::pin!(action.run(context, &self.qemu_args, self.accel.value));
        let result = loop {
            tokio::select! {
                biased;
                result = &mut run => break result,
                true = async { server.accept_client() } => {}
                () = tokio::time::sleep(ACCEPT_INTERVAL) => {}
            }
        };
        result.map(|_| 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::ContextBuilder;
    use crate::error::Error;
    use crate::models::{Arch, Instance};
    use crate::platform::SystemMock;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_runs_qemu_for_the_instance() {
        let system = SystemMock::new().add_env_var("CUBIC_QEMU_FW_AMD64", "/fw/OVMF.fd");
        let builder = ContextBuilder::new(system);
        builder.add_instances(vec![Instance {
            name: "test".to_string(),
            arch: Arch::AMD64,
            ..Instance::default()
        }]);
        let system = Arc::clone(builder.get_system());
        let args = ["vm", "--accel", "off", "--qemu-args=-sandbox on", "test"];
        let command = VmCommand::try_parse_from(args).unwrap();

        let result = command.run(&builder.build()).await;

        assert!(matches!(result, Err(Error::QemuNotFound)));
        let executed = system.get_executed_commands();
        assert_eq!(executed.len(), 1);
        assert!(executed[0].starts_with("qemu-system-x86_64 "));
        assert!(executed[0].contains("-accel tcg"));
        assert!(executed[0].contains("-sandbox on"));
    }
}
