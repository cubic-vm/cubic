use crate::error::Error;
use crate::instance::InstanceDao;
use crate::util;

pub fn info(instance_dao: &InstanceDao, instance: String) -> Result<(), Error> {
    if !instance_dao.exists(&instance) {
        return Result::Err(Error::UnknownInstance(instance.clone()));
    }

    let instance = instance_dao.load(&instance)?;
    let cpus = &instance.cpus;
    let mem = util::bytes_to_human_readable(instance.mem);
    let disk = util::bytes_to_human_readable(instance.disk_capacity);
    let user = &instance.user;
    let display = &instance.display;
    let gpu = &instance.gpu;
    let port = &instance.ssh_port;

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

    for mount in &instance.mounts {
        println!("  - {} => {}", mount.host, mount.guest);
    }

    println!("hostfwd:");
    for rule in &instance.hostfwd {
        println!("  - {rule}");
    }

    Result::Ok(())
}
