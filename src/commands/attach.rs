use crate::error::Error;
use crate::machine::MachineDao;
use std::io;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;
use std::str;

pub fn attach(machine_dao: &MachineDao, instance: &str) -> Result<(), Error> {
    let machine = machine_dao.load(instance)?;
    if !machine_dao.is_running(&machine) {
        return Result::Err(Error::MachineNotRunning(machine.name.to_string()));
    }

    let mut stream = UnixStream::connect(format!(
        "{}/{}/console.sock",
        &machine_dao.cache_dir, instance
    ))
    .map_err(|_| Error::CannotAttach(instance.to_string()))?;
    let buf = &mut [0_u8; 1024];

    loop {
        if let Result::Ok(length) = stream.read(buf) {
            io::stdout().write_all(&buf[0..length]).unwrap();
        }
    }
}
