use crate::actions::{LoadInstanceAction, StopInstanceAction};
use crate::commands::{self, Command};
use crate::error::Result;
use crate::models::Target;
use crate::view::Console;
use clap::{self, ArgAction, Parser};
use std::sync::Arc;

/// Create and start a VM instance
///
/// This command is a shortcut for the three subcommands `create`, `start` and `ssh`.
///
/// Examples:
///
///   Run a VM instance with 8 vCPUs, 10G of RAM, 200G of storage:
///   $ cubic run example1 --cpus 8 --memory 10G --disk 200G -i debian:trixie
///
///   Run a VM instance and forward the instance's HTTP port to the host port 8000:
///   $ cubic run example2 --port 8000:80 -i ubuntu
///
///   Run a VM instance and forward the instance's DNS port to the host port 5353:
///   $ cubic run example3 --port 5353:53/udp -i ubuntu
///
///   Run a VM instance with multiple port forwarding rules:
///   $ cubic run example4 -p 8000:80/tcp -p 5353:53/udp -i ubuntu:latest
///
///   Run a VM instance and install Vim:
///   $ cubic run example5 -e "sudo apt install -y vim" -i ubuntu
///
///   Run a VM instance without network access:
///   $ cubic run example6 --isolate -i ubuntu
///
///   Run a VM instance and delete it when you exit:
///   $ cubic run --rm example7 -i debian:trixie
///
///   Every distribution has the tags latest and stable. The tag latest is the
///   newest release and the tag stable is the newest long term release. A plain
///   name is a shortcut for stable, so --image ubuntu gives you the last LTS.
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct RunCommand {
    #[clap(flatten)]
    create_cmd: commands::CreateCommand,
    /// Delete the VM instance when the session ends
    #[clap(long, action = ArgAction::SetTrue)]
    rm: bool,
    #[clap(flatten)]
    accel: commands::AccelArg,
    #[clap(flatten)]
    env_args: commands::EnvArgs,
}

impl RunCommand {
    // Best effort, a failure here must not mask the session result.
    fn cleanup(&self, console: &Arc<Console>, context: &commands::Context) {
        let name = self.create_cmd.instance_name.value.as_str();
        let store = context.get_instance_store();

        if let Ok(instance) = LoadInstanceAction::new().run(context, console, name) {
            StopInstanceAction::new(&instance).run(store, true).ok();
            store.delete(&instance).ok();
        }
    }
}

impl Command for RunCommand {
    async fn run(&self, console: &Arc<Console>, context: &commands::Context) -> Result<u8> {
        self.create_cmd.create(console, context, self.rm).await?;

        let ssh = commands::SshCommand {
            target: Target::from_instance_name(self.create_cmd.instance_name.value.clone()),
            accel: self.accel,
            env_args: self.env_args.clone(),
        };
        // Ctrl+C ends the session like an exit. The dropped shell cannot
        // reset the terminal, so reset it here.
        let result = tokio::select! {
            result = ssh.run(console, context) => result,
            _ = tokio::signal::ctrl_c() => {
                console.reset();
                Ok(130)
            }
        };

        if self.rm {
            self.cleanup(console, context);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::InstanceStoreMock;
    use crate::models::{Environment, Instance};
    use crate::platform::SystemMock;
    use std::sync::Arc;

    #[test]
    fn test_rm_stops_and_deletes_the_instance() {
        let system = SystemMock::new();
        let console = &Console::new(Arc::new(system));
        let store = InstanceStoreMock::new_with_running(
            vec![Instance {
                name: "web".to_string(),
                ..Instance::default()
            }],
            &["web"],
        );
        let killed = Arc::clone(&store.killed);
        let deleted = Arc::clone(&store.deleted);
        let context = commands::Context::new(
            Arc::new(SystemMock::new()),
            Environment::default(),
            Box::new(store),
        );

        RunCommand::try_parse_from(["run", "--rm", "web", "-i", "debian:trixie"])
            .unwrap()
            .cleanup(console, &context);

        assert_eq!(*killed.lock().unwrap(), ["web"]);
        assert_eq!(*deleted.lock().unwrap(), ["web"]);
    }
}
