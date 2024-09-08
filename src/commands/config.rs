use crate::error::Error;
use crate::machine::MachineDao;
use crate::util;

pub fn config(
    machine_dao: &MachineDao,
    instance: &str,
    cpus: &Option<u16>,
    mem: &Option<String>,
    disk: &Option<String>,
    sandbox: &Option<bool>,
) -> Result<(), Error> {
    let mut machine = machine_dao.load(instance)?;

    if let Some(cpus) = cpus {
        machine.cpus = *cpus;
    }

    if let Some(mem) = mem {
        machine.mem = util::human_readable_to_bytes(mem)?;
    }

    if let Some(disk) = disk {
        machine_dao.resize(&mut machine, util::human_readable_to_bytes(disk)?)?;
    }

    if let Some(sandbox) = sandbox {
        machine.sandbox = *sandbox;
    }

    machine_dao.store(&machine)?;
    println!("cpus:    {}", machine.cpus);
    println!("mem:     {}", util::bytes_to_human_readable(machine.mem));
    println!(
        "disk:    {}",
        util::bytes_to_human_readable(machine.disk_capacity)
    );
    println!("user:    {}", machine.user);
    println!("mounts:");
    for mount in &machine.mounts {
        println!("  - {} => {}", mount.host, mount.guest);
    }
    println!("display: {}", machine.display);
    println!("gpu: {}", machine.gpu);
    println!("sandbox: {}", machine.sandbox);
    println!("ssh-port: {}", machine.ssh_port);
    Result::Ok(())
}
