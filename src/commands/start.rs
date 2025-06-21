use crate::commands::Verbosity;
use crate::error::Error;
use crate::instance::{InstanceDao, InstanceState, InstanceStore};
use crate::view::SpinnerView;
use std::io::Read;
use std::thread;
use std::time::Duration;

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
        let mut all_running: bool = true;
        let mut any_fails: bool = true;
        while all_running || any_fails {
            thread::sleep(Duration::from_secs(1));

            all_running = instances
                .iter()
                .all(|instance| instance_dao.get_state(instance) == InstanceState::Running);
            any_fails = children.iter_mut().any(|child| {
                child
                    .try_wait()
                    .ok()
                    .and_then(|result| result)
                    .and_then(|exit| exit.code())
                    .map(|exit_code| exit_code != 0)
                    .unwrap_or_default()
            });
        }
        spinner.stop();

        for mut child in children {
            let exit_code = child
                .try_wait()
                .ok()
                .and_then(|result| result)
                .and_then(|exit| exit.code());

            if let Some(exit_code) = exit_code {
                if exit_code != 0 {
                    let mut stderr = String::new();
                    if let Some(mut err) = child.stderr.take() {
                        err.read_to_string(&mut stderr).ok();
                    }

                    let message = format!("QEMU failed with exit code {exit_code}:\n{stderr}");
                    return Err(Error::CommandFailed(message));
                }
            }
        }
    }

    Result::Ok(())
}
