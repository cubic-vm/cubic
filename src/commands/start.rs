use crate::commands::Verbosity;
use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore};
use crate::ssh_cmd::PortChecker;
use crate::view::SpinnerView;

pub fn start(
    instance_dao: &InstanceDao,
    qemu_args: &Option<String>,
    verbosity: Verbosity,
    instance_names: &Vec<String>,
) -> Result<(), Error> {
    for name in instance_names {
        if !instance_dao.exists(name) {
            return Result::Err(Error::UnknownInstance(name.clone()));
        }
    }

    let mut instances = Vec::new();
    let mut children = Vec::new();
    for name in instance_names {
        let instance = instance_dao.load(name)?;
        if !instance_dao.is_running(&instance) {
            let child = instance_dao.start(&instance, qemu_args, verbosity.is_verbose())?;
            children.push(child);
        }
        instances.push(instance);
    }

    if !verbosity.is_quiet() {
        let mut spinner = SpinnerView::new("Starting instance(s)");
        while !instances
            .iter()
            .all(|instance| PortChecker::new(instance.ssh_port).try_connect())
        {
            for ref mut child in children.iter_mut() {
                if child
                    .try_wait()
                    .ok()
                    .and_then(|result| result)
                    .map(|exit| !exit.success())
                    .unwrap_or_default()
                {
                    spinner.stop();
                    return Err(Error::CommandFailed("QEMU failed".to_string()));
                }
            }
        }
        spinner.stop();
    }

    Result::Ok(())
}
