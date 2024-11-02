use crate::error::Error;
use crate::machine::MachineDao;
use crate::util;

pub fn info(machine_dao: &MachineDao, instance: String) -> Result<(), Error> {
    if !machine_dao.exists(&instance) {
        return Result::Err(Error::UnknownMachine(instance.clone()));
    }

    let machine = machine_dao.load(&instance)?;
    let cpus = &machine.cpus;
    let mem = util::bytes_to_human_readable(machine.mem);
    let disk = util::bytes_to_human_readable(machine.disk_capacity);
    let user = &machine.user;
    let display = &machine.display;
    let gpu = &machine.gpu;
    let port = &machine.ssh_port;

    print!(
        "\
        cpus:     {cpus}\n\
        mem:      {mem}\n\
        disk:     {disk}\n\
        user:     {user}\n\
        display:  {display}\n\
        gpu:      {gpu}\n\
        ssh-port: {port}\n\
        mounts:\n\
    "
    );

    for mount in &machine.mounts {
        println!("  - {} => {}", mount.host, mount.guest);
    }

    Result::Ok(())
}
