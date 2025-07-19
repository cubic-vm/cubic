use crate::commands::Verbosity;
use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore};
use crate::ssh_cmd::PortChecker;
use crate::view::SpinnerView;
use std::thread::sleep;
use std::time::Duration;

pub fn start(
    instance_dao: &InstanceDao,
    qemu_args: &Option<String>,
    verbosity: Verbosity,
    instance_names: &Vec<String>,
) -> Result<(), Error> {
    let mut spinner = (!verbosity.is_quiet()).then(|| SpinnerView::new("Starting instance(s)"));

    for name in instance_names {
        if !instance_dao.exists(name) {
            if let Some(ref mut spinner) = &mut spinner {
                spinner.stop()
            }
            return Result::Err(Error::UnknownInstance(name.clone()));
        }
    }

    let mut instances = Vec::new();
    for name in instance_names {
        let instance = instance_dao.load(name)?;
        if !instance_dao.is_running(&instance) {
            let result = instance_dao.start(&instance, qemu_args, verbosity.is_verbose());
            if result.is_err() {
                if let Some(ref mut spinner) = &mut spinner {
                    spinner.stop()
                }
            }
            result?;
        }
        instances.push(instance);
    }

    if !verbosity.is_quiet() {
        while !instances
            .iter()
            .all(|instance| PortChecker::new(instance.ssh_port).try_connect())
        {
            sleep(Duration::from_secs(1));
        }
    }

    if let Some(ref mut spinner) = &mut spinner {
        spinner.stop()
    }

    Result::Ok(())
}
