mod actions;
mod arch;
mod commands;
mod emulator;
mod env;
mod error;
mod fs;
mod image;
mod instance;
mod qemu;
mod ssh_cmd;
mod util;
mod view;
mod web;

use crate::commands::CommandDispatcher;

fn main() {
    CommandDispatcher::new()
        .dispatch()
        .map_err(error::print_error)
        .ok();
}
