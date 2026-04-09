use crate::actions::StartInstanceAction;
use crate::commands::{self, Command};
use crate::error::Result;
use crate::ssh_cmd::{PortChecker, Russh};
use crate::util;
use crate::view::Console;
use crate::view::SpinnerView;
use clap::Parser;
use std::thread::sleep;
use std::time::Duration;

/// Start VM instances
///
/// Examples:
///
///   Start the VM instance 'my-instance'
///   $ cubic start my-instance
///
///   Start and wait for the VM instance 'my-instance' to start
///   $ cubic start --wait my-instance
///
///   Start multiple VM instances
///   $ cubic start trixie noble
///
///   Pass additional arguments to QEMU
///   $ cubic start trixie --qemu-args="-sandbox on"
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct StartCommand {
    /// Pass additional QEMU arguments
    #[clap(long)]
    pub qemu_args: Option<String>,
    /// Wait until the VM instance has started
    #[clap(short, long, default_value_t = false)]
    pub wait: bool,
    /// Name of the virtual machine instances to start
    pub instances: Vec<String>,
}

impl Command for StartCommand {
    fn run(&self, console: &mut dyn Console, context: &commands::Context) -> Result<()> {
        let instance_store = context.get_instance_store();

        let verbosity = console.get_verbosity();
        let async_caller = util::AsyncCaller::new();
        let russh = Russh::new();

        // Launch virtual machine instances
        let mut actions = Vec::new();
        for name in &self.instances {
            let instance = &mut instance_store.load(name)?;
            if !async_caller.call(russh.is_running(instance.ssh_port)) {
                // Make SSH port is available
                if PortChecker::new().is_open(instance.ssh_port) {
                    instance.ssh_port = PortChecker::new().get_new_port();
                    instance_store.store(instance)?;
                }

                let mut action = StartInstanceAction::new(instance);
                action.run(context, &self.qemu_args, verbosity.is_verbose())?;

                actions.push(action);
            }
        }

        // Wait for virtual machine instances to be started
        if self.wait && !verbosity.is_quiet() {
            let mut spinner = SpinnerView::new("Starting instance(s)".to_string());
            while actions.iter().any(|a| !a.is_done()) {
                sleep(Duration::from_secs(1));
            }
            spinner.stop()
        }

        Result::Ok(())
    }
}
