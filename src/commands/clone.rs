use crate::error::Error;
use crate::machine::MachineDao;
use crate::util::generate_random_ssh_port;

pub fn clone(machine_dao: &MachineDao, name: &str, new_name: &str) -> Result<(), Error> {
    machine_dao.clone(&machine_dao.load(name)?, new_name)?;

    let mut new_machine = machine_dao.load(new_name)?;
    new_machine.ssh_port = generate_random_ssh_port();
    machine_dao.store(&new_machine)
}
