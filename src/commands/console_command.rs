use crate::actions::LoadInstanceAction;
use crate::commands::{self, Command};
use crate::error::{Error, Result};
use crate::util;
use clap::Parser;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;
use tokio_util::codec::FramedRead;
use tokio_util::io::{StreamReader, SyncIoBridge};

const CONSOLE_TIMEOUT: Duration = Duration::from_secs(60);

/// Open VM instance console
///
/// Examples:
///
///   Connect to the console of 'my-instance'
///   $ cubic console my-instance
///   Login requires a password. Set one with 'sudo passwd' over cubic ssh.
///   Press Enter, ~, . to exit the console.
///
///   [...]
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct ConsoleCommand {
    #[clap(flatten)]
    pub accel: commands::AccelArg,
    #[clap(flatten)]
    instance: commands::InstanceArg,
}

impl Command for ConsoleCommand {
    async fn run(&self, context: &commands::Context) -> Result<u8> {
        let console = context.get_console();
        commands::StartCommand {
            qemu_args: None,
            accel: self.accel,
            wait: false,
            yes: commands::YesArg { value: false },
            instances: self.instance.value.clone().into(),
        }
        .run(context)
        .await?;

        let instance = LoadInstanceAction::new().run(context, self.instance.value.as_str())?;

        console.info(&format!(
            "Login requires a password. Set one with '{} passwd' over cubic ssh.",
            instance.get_privilege_tool()
        ));
        console.info("Press Enter, ~, . to exit the console.");

        let socket_path = context.get_env().get_console_socket(&instance.name);

        let system = context.get_system();
        let instance_store = context.get_instance_store();
        let wait = async {
            let mut was_running = false;
            loop {
                if let Ok(socket) = system.connect_socket(Path::new(&socket_path), None) {
                    return Ok(socket);
                }
                // QEMU writes its pid file after the spawn returns, so only a pid
                // file that vanished again means it exited.
                let running = instance_store.is_running(&instance);
                if was_running && !running {
                    return Err(Error::InstanceNotRunning(instance.name.clone()));
                }
                was_running |= running;

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        };
        let mut socket = tokio::time::timeout(CONSOLE_TIMEOUT, wait)
            .await
            .map_err(|_| Error::ConsoleTimeout(instance.name.clone()))??;

        let mut reader = socket.try_clone()?;

        console.raw_mode();
        // The socket read blocks, so it runs on its own thread
        let output = tokio::task::spawn_blocking(move || {
            let mut stdout = std::io::stdout();
            let mut buffer = [0; 4096];
            while let Ok(count @ 1..) = reader.read(&mut buffer) {
                if stdout.write_all(&buffer[..count]).is_err() || stdout.flush().is_err() {
                    break;
                }
            }
        });
        let stdin = StreamReader::new(FramedRead::new(
            tokio::io::stdin(),
            util::ShortcutDecoder::new(),
        ));
        let input = tokio::task::spawn_blocking(move || {
            std::io::copy(&mut SyncIoBridge::new(stdin), &mut socket)
        });
        tokio::select!(
            _ = input => {},
            _ = output => {},
        );

        let mut stdout = std::io::stdout();
        stdout.write_all(b"\n").ok();
        stdout.flush().ok();
        console.reset();
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reject_path_traversal() {
        assert!(ConsoleCommand::try_parse_from(["console", "../../etc"]).is_err());
    }
}
