use crate::error::Error;
use crate::machine::{MachineDao, MountPoint};
use crate::util;

pub fn mount(machine_dao: &MachineDao, name: &str, host: &str, guest: &str) -> Result<(), Error> {
    let mut machine = machine_dao.load(name)?;
    machine.mounts.push(MountPoint {
        host: host.to_string(),
        guest: guest.to_string(),
    });
    machine_dao.store(&machine)?;
    util::setup_cloud_init(&machine, &machine_dao.cache_dir, true)
}
