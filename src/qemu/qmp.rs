use crate::commands::Verbosity;
use crate::error::{Error, Result};
use crate::qemu;
use serde_json::Value;
use std::io::{BufRead, BufReader, Read, Write};

trait ReadWrite: Read + Write {}
impl<T: Read + Write> ReadWrite for T {}

pub struct Qmp {
    counter: u64,
    verbosity: Verbosity,
    stream: BufReader<Box<dyn ReadWrite>>,
}

impl Qmp {
    pub fn new(stream: impl Read + Write + 'static, verbosity: Verbosity) -> Self {
        Qmp {
            counter: 0,
            verbosity,
            stream: BufReader::new(Box::new(stream)),
        }
    }

    pub fn send(&mut self, message: &qemu::QmpMessage) -> Result<()> {
        let request = serde_json::to_string(message).map_err(Error::from)?;

        if self.verbosity.is_verbose() {
            println!("QMP send: {request}");
        }
        self.stream
            .get_mut()
            .write_all(request.as_bytes())
            .map_err(Error::from)?;
        self.stream.get_mut().flush().map_err(Error::from)
    }

    pub fn recv(&mut self) -> Result<qemu::QmpMessage> {
        let mut response = String::new();
        self.stream.read_line(&mut response).map_err(Error::from)?;

        if self.verbosity.is_verbose() {
            println!("QMP recv: {response}");
        }

        serde_json::from_str(&response).map_err(Error::from)
    }

    pub fn execute_with_args(&mut self, cmd: &str, arguments: Value) -> Result<qemu::QmpMessage> {
        let request_id = Some(self.counter.to_string());
        self.counter += 1;

        self.send(&qemu::QmpMessage::Command {
            id: request_id.clone(),
            execute: cmd.to_string(),
            arguments,
        })?;

        loop {
            let response = self.recv()?;
            match &response {
                qemu::QmpMessage::Success { id, .. } | qemu::QmpMessage::Error { id, .. }
                    if *id == request_id =>
                {
                    return Ok(response);
                }
                _ => {}
            }
        }
    }

    pub fn execute(&mut self, cmd: &str) -> Result<()> {
        self.execute_with_args(cmd, Value::Null).map(|_| ())
    }
}
