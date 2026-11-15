use crate::error::{Error, Result};
use crate::platform::Process;
use crate::util::CommandLine;
use std::process::Command;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const TIMED_OUT: &str = "The command timed out.";

pub struct SystemCommand {
    command: Command,
}

impl SystemCommand {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub async fn run(
        self,
        process: &dyn Process,
        stdin: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<Vec<u8>> {
        let line = self.command.get_command();
        let mut child = process.spawn(self.command, false)?;
        let mut input = child.take_stdin();
        let mut stdout = child.take_stdout();
        let mut stderr = child.take_stderr();
        let (mut out, mut err) = (Vec::new(), Vec::new());

        // The write owns the pipe, so finishing it sends EOF.
        let write = async move { input.write_all(stdin.unwrap_or_default().as_bytes()).await };
        let run = async {
            let read_out = stdout.read_to_end(&mut out);
            let read_err = stderr.read_to_end(&mut err);
            tokio::join!(write, read_out, read_err, child.wait()).3
        };

        match tokio::time::timeout(timeout.unwrap_or(Duration::MAX), run).await {
            Err(_) => Err(Error::SystemCommandFailed(line, TIMED_OUT.to_string())),
            Ok(true) => Ok(out),
            Ok(false) => {
                let stderr = String::from_utf8_lossy(&err).into_owned();
                Err(Error::SystemCommandFailed(line, stderr))
            }
        }
    }
}
