use crate::error::Error;

use std::io::{prelude::*, BufReader, BufWriter};
use std::os::unix::net::UnixStream;

pub struct Monitor {
    reader: BufReader<UnixStream>,
    writer: BufWriter<UnixStream>,
}

impl Monitor {
    pub fn new(path: &str) -> Result<Self, Error> {
        let socket = UnixStream::connect(path).map_err(Error::Io)?;

        let mut monitor = Monitor {
            reader: BufReader::new(socket.try_clone().map_err(Error::Io)?),
            writer: BufWriter::new(socket.try_clone().map_err(Error::Io)?),
        };

        monitor.init()?;
        Ok(monitor)
    }

    fn read_line(&mut self) -> Result<String, Error> {
        let mut response = String::new();
        self.reader.read_line(&mut response).map_err(Error::Io)?;
        Ok(response)
    }

    fn execute(&mut self, cmd: &str) -> Result<String, Error> {
        self.writer
            .write(format!("{{ \"execute\": \"{cmd}\" }}\n").as_bytes())
            .map_err(Error::Io)?;
        self.writer.flush().map_err(Error::Io)?;
        self.read_line()
    }

    fn init(&mut self) -> Result<(), Error> {
        self.read_line().map(|_| ())?;
        self.execute("qmp_capabilities").map(|_| ())
    }

    pub fn shutdown(&mut self) -> Result<(), Error> {
        self.execute("system_powerdown").map(|_| ())
    }
}
