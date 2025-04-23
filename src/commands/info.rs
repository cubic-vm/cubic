use crate::error::Error;
use crate::instance::InstanceDao;
use crate::util;
use crate::view::MapView;

pub fn info(instance_dao: &InstanceDao, instance: String) -> Result<(), Error> {
    if !instance_dao.exists(&instance) {
        return Result::Err(Error::UnknownInstance(instance.clone()));
    }

    let instance = instance_dao.load(&instance)?;

    let mut view = MapView::new();
    view.add("CPUs", &instance.cpus.to_string());
    view.add("Memory", &util::bytes_to_human_readable(instance.mem));
    view.add(
        "Disk",
        &util::bytes_to_human_readable(instance.disk_capacity),
    );
    view.add("User", &instance.user);
    view.add("Display", &instance.display.to_string());
    view.add("GPU", &instance.gpu.to_string());
    view.add("SSH Port", &instance.ssh_port.to_string());

    for (index, mount) in instance.mounts.iter().enumerate() {
        let key = if index == 0 { "Mounts" } else { "" };
        view.add(key, &format!("{} => {}", mount.host, mount.guest));
    }

    for (index, rule) in instance.hostfwd.iter().enumerate() {
        let key = if index == 0 { "Forward" } else { "" };
        view.add(key, rule);
    }

    view.print();

    Ok(())
}
