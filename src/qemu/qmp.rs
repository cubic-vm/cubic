use crate::commands::Verbosity;
use crate::error::Error;
use std::collections::HashMap;
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

    pub fn read_line(&mut self) -> Result<String, Error> {
        let mut response = String::new();
        self.reader.read_line(&mut response).map_err(Error::Io)?;

        if self.verbosity.is_verbose() {
            print!("Guest-Agent Server: {response}");
        }
        Ok(response)
    }

    pub fn execute_with_args(
        &mut self,
        cmd: &str,
        args: &HashMap<String, String>,
    ) -> Result<String, Error> {
        let args = args
            .iter()
            .map(|(key, value)| format!("\"{key}\": {value}"))
            .collect::<Vec<_>>()
            .join(", ");

        let request = format!("{{ \"execute\": \"{cmd}\", \"arguments\": {{{args}}} }}\n");
        if self.verbosity.is_verbose() {
            print!("Guest-Agent Client: {request}");
        }
        self.writer.write(request.as_bytes()).map_err(Error::Io)?;
        self.writer.flush().map_err(Error::Io)?;
        self.read_line()
    }

    pub fn execute(&mut self, cmd: &str) -> Result<String, Error> {
        self.execute_with_args(cmd, &HashMap::new())
    }
}
