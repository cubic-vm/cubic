use crate::commands::Verbosity;
use crate::error::Error;
use crate::qemu;
use serde_json::Value;
use std::io::{prelude::*, BufReader, BufWriter, Read, Write};
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
    pub fn new(path: &str, verbosity: Verbosity) -> Result<Self, Error> {
        let socket = UnixStream::connect(path).map_err(Error::Io)?;

        let get_timeout = || Some(Duration::from_millis(QMP_TIMEOUT_MS));
        socket.set_read_timeout(get_timeout()).map_err(Error::Io)?;
        socket.set_write_timeout(get_timeout()).map_err(Error::Io)?;

        Ok(Qmp {
            counter: 0,
            verbosity,
            reader: BufReader::new(Box::new(socket.try_clone().map_err(Error::Io)?)),
            writer: BufWriter::new(Box::new(socket.try_clone().map_err(Error::Io)?)),
        })
    }

    pub fn send(&mut self, message: &qemu::QmpMessage) -> Result<(), Error> {
        let request = serde_json::to_string(message).map_err(Error::SerdeJson)?;

        if self.verbosity.is_verbose() {
            println!("QMP send: {request}");
        }
        self.writer.write(request.as_bytes()).map_err(Error::Io)?;
        self.writer.flush().map_err(Error::Io)
    }

    pub fn recv(&mut self) -> Result<qemu::QmpMessage, Error> {
        let mut response = String::new();
        self.reader.read_line(&mut response).map_err(Error::Io)?;

        if self.verbosity.is_verbose() {
            println!("QMP recv: {response}");
        }

        serde_json::from_str(&response).map_err(Error::SerdeJson)
    }

    pub fn execute_with_args(
        &mut self,
        cmd: &str,
        arguments: Value,
    ) -> Result<qemu::QmpMessage, Error> {
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
                    return Ok(response)
                }
                _ => {}
            }
        }
    }

    pub fn execute(&mut self, cmd: &str) -> Result<(), Error> {
        self.execute_with_args(cmd, Value::Null).map(|_| ())
    }
}
