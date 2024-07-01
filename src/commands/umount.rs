use crate::error::Error;
use crate::machine::MachineDao;
use crate::util;

pub fn umount(machine_dao: &MachineDao, name: &str, guest: &str) -> Result<(), Error> {
    let mut machine = machine_dao.load(name)?;
    machine.mounts.retain(|mount| mount.guest != guest);
    machine_dao.store(&machine)?;
    util::setup_cloud_init(&machine, &machine_dao.cache_dir, true)
}
