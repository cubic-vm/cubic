use crate::commands::{self, Command};
use crate::error::{Error, Result};
use crate::models::InstanceCertPaths;
use crate::qemu::TlsClient;
use crate::util;
use crate::view::Console;
use clap::Parser;
use std::net::TcpStream;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio_util::codec::FramedRead;
use tokio_util::io::StreamReader;

/// Open VM instance console
///
/// Examples:
///
///   Connect to the console of 'my-instance'
///   $ cubic console my-instance
///   Default credentials: cubic / cubic
///   Press Enter, ~, . to exit the console.
///
///   [...]
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct ConsoleCommand {
    #[clap(flatten)]
    instance: commands::InstanceArg,
}

impl Command for ConsoleCommand {
    fn run(&self, console: &mut Console<'_>, context: &commands::Context) -> Result<()> {
        commands::StartCommand {
            qemu_args: None,
            wait: false,
            yes: commands::YesArg { value: false },
            instances: self.instance.value.clone().into(),
        }
        .run(console, context)?;

        let instance = context
            .get_instance_store()
            .load(self.instance.value.as_str())?;

        console.info(&format!("Default credentials: {} / cubic", instance.user));
        console.info("Press Enter, ~, . to exit the console.");

        let port = instance
            .console_port
            .ok_or_else(|| Error::InstanceNotRunning(self.instance.value.to_string()))?;
        let instance_dir = PathBuf::from(
            context
                .get_env()
                .get_instance_dir2(self.instance.value.as_str()),
        );
        let certs = InstanceCertPaths::load(&instance_dir);

        while TcpStream::connect(format!("127.0.0.1:{port}")).is_err() {
            thread::sleep(Duration::from_secs(1));
        }

        console.raw_mode();
        let shell = util::AsyncCaller::new().call(async {
            let tls = TlsClient::new(&certs)?.connect_async(port).await?;
            let (mut reader, mut writer) = tokio::io::split(tls);
            let mut stdin = StreamReader::new(FramedRead::new(
                tokio::io::stdin(),
                util::ShortcutDecoder::new(),
            ));
            let mut stdout = tokio::io::stdout();
            tokio::select!(
                _ = tokio::io::copy(&mut stdin, &mut writer) => {},
                _ = tokio::io::copy(&mut reader, &mut stdout) => {},
            );

            let mut out = tokio::io::stdout();
            out.write_all(b"\n").await.ok();
            out.flush().await.ok();
            Ok::<(), Error>(())
        });
        if shell.is_err() {
            console.error("Cannot open shell");
        }
        console.reset();
        Ok(())
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
