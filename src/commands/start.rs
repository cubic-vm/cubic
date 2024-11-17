use crate::commands::Verbosity;
use crate::error::Error;
use crate::machine::{MachineDao, MachineState};
use crate::view::TimerView;
use std::io::Read;

pub fn start(
    machine_dao: &MachineDao,
    qemu_args: &Option<String>,
    verbosity: Verbosity,
    instances: &Vec<String>,
) -> Result<(), Error> {
    for instance in instances {
        if !machine_dao.exists(instance) {
            return Result::Err(Error::UnknownMachine(instance.clone()));
        }
    }

    let mut machines = Vec::new();
    let mut children = Vec::new();
    for instance in instances {
        let machine = machine_dao.load(instance)?;
        if !machine_dao.is_running(&machine) {
            let child = machine_dao.start(&machine, qemu_args, verbosity.is_verbose())?;
            children.push(child);
        }
        machines.push(machine);
    }

    if !verbosity.is_quiet() {
        TimerView::new("Starting instance(s)").run(&mut || {
            let all_running = machines
                .iter()
                .all(|machine| machine_dao.get_state(machine) == MachineState::Running);
            let any_fails = children.iter_mut().any(|child| {
                child
                    .try_wait()
                    .ok()
                    .and_then(|result| result)
                    .and_then(|exit| exit.code())
                    .map(|exit_code| exit_code != 0)
                    .unwrap_or_default()
            });

            all_running || any_fails
        });

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
