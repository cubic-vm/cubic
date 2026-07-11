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
mod qemu;
mod ssh_cmd;
mod util;
mod view;
mod web;

use crate::commands::CommandDispatcher;
use crate::view::Console;
use clap::Parser;
use std::process::ExitCode;

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

    let console = &mut view::Stdio::new();
    match CommandDispatcher::parse().dispatch(console) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            console.error(&e.to_string());
            ExitCode::FAILURE
        }
    }
}
