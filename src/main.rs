mod actions;
mod cloudinit;
mod commands;
mod env;
mod error;
mod fs;
mod image;
mod instance;
mod iso9660;
mod models;
mod platform;
mod qemu;
mod ssh_cmd;
mod util;
mod view;
mod web;

use crate::commands::CommandDispatcher;
use crate::platform::{OsSystem, System};
use clap::Parser;
use std::process::ExitCode;
use std::rc::Rc;

fn main() -> ExitCode {
    // Disable raw mode before the default panic hook runs, so a panic during
    // an interactive session (ssh, console) does not leave the terminal
    // broken. This also covers panic = 'abort' builds, since the hook runs
    // before the process aborts.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        crossterm::terminal::disable_raw_mode().ok();
        default_hook(info);
    }));

    let system: Rc<dyn System> = Rc::new(OsSystem::new());
    let console = &mut view::Console::new(system.as_ref());
    match CommandDispatcher::parse().dispatch(Rc::clone(&system), console) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            console.error(&e.to_string());
            ExitCode::FAILURE
        }
    }
}
