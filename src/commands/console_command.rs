use crate::commands::{self, Command};
use crate::error::{Error, Result};
use crate::models::InstanceCertPaths;
use crate::qemu::TlsClient;
use crate::util::Terminal;
use crate::view::Console;
use clap::Parser;
use std::net::TcpStream;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

/// Open VM instance console
///
/// Examples:
///
///   Connect to the console of 'my-instance'
///   $ cubic console my-instance
///   Default credentials: cubic / cubic
///   Press CTRL+W to exit the console.
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
    fn run(&self, console: &mut dyn Console, context: &commands::Context) -> Result<()> {
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
        console.info("Press CTRL+W to exit the console.");

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
        match TlsClient::new(&certs).and_then(|c| c.connect(port)) {
            Ok(mut tls) => {
                tls.get_mut()
                    .set_read_timeout(Some(Duration::from_millis(10)))
                    .ok();
                Terminal::open(tls).wait();
            }
            Err(_) => console.error("Cannot open shell"),
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
