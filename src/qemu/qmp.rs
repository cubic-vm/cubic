use crate::commands::Verbosity;
use crate::error::Error;
use crate::qemu;
use serde_json::Value;
use std::io::{prelude::*, BufReader, BufWriter, Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

pub struct Qmp {
    verbosity: Verbosity,
    reader: BufReader<Box<dyn Read>>,
    writer: BufWriter<Box<dyn Write>>,
}

impl Qmp {
    pub fn new(path: &str, verbosity: Verbosity) -> Result<Self, Error> {
        let socket = UnixStream::connect(path).map_err(Error::Io)?;
        socket
            .set_read_timeout(Some(Duration::from_millis(50)))
            .map_err(Error::Io)?;

        Ok(Qmp {
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

    pub fn execute_with_args(&mut self, cmd: &str, arguments: Value) -> Result<(), Error> {
        self.send(&qemu::QmpMessage::Command {
            id: None,
            execute: cmd.to_string(),
            arguments,
        })?;
        self.recv().map(|_| ())
    }

    pub fn execute(&mut self, cmd: &str) -> Result<(), Error> {
        self.execute_with_args(cmd, Value::Null).map(|_| ())
    }
}
