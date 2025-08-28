mod actions;
mod arch;
mod commands;
mod emulator;
mod env;
mod error;
mod fs;
mod image;
mod instance;
mod model;
mod qemu;
mod ssh_cmd;
mod util;
mod view;
mod web;

use crate::commands::CommandDispatcher;
use clap::Parser;

fn main() {
    let console = &mut view::Stdio::new();
    CommandDispatcher::parse()
        .dispatch(console)
        .map_err(|e| e.print(console))
        .ok();
}
