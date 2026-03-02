use crate::commands::Verbosity;
use crate::error::{Error, Result};
use crate::qemu;
use serde_json::Value;
use std::io::{BufReader, BufWriter, Read, Write, prelude::*};
use std::os::unix::net::UnixStream;
use std::time::Duration;

const QMP_TIMEOUT_MS: u64 = 100;

pub struct Qmp {
    counter: u64,
    verbosity: Verbosity,
    reader: BufReader<Box<dyn Read>>,
    writer: BufWriter<Box<dyn Write>>,
}

impl Qmp {
    pub fn new(path: &str, verbosity: Verbosity) -> Result<Self> {
        let socket = UnixStream::connect(path).map_err(Error::from)?;

        let get_timeout = || Some(Duration::from_millis(QMP_TIMEOUT_MS));
        socket
            .set_read_timeout(get_timeout())
            .map_err(Error::from)?;
        socket
            .set_write_timeout(get_timeout())
            .map_err(Error::from)?;

        Ok(Qmp {
            counter: 0,
            verbosity,
            reader: BufReader::new(Box::new(socket.try_clone().map_err(Error::from)?)),
            writer: BufWriter::new(Box::new(socket.try_clone().map_err(Error::from)?)),
        })
    }

    pub fn send(&mut self, message: &qemu::QmpMessage) -> Result<()> {
        let request = serde_json::to_string(message).map_err(Error::from)?;

        if self.verbosity.is_verbose() {
            println!("QMP send: {request}");
        }
        self.writer.write(request.as_bytes()).map_err(Error::from)?;
        self.writer.flush().map_err(Error::from)
    }

    pub fn recv(&mut self) -> Result<qemu::QmpMessage> {
        let mut response = String::new();
        self.reader.read_line(&mut response).map_err(Error::from)?;

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
