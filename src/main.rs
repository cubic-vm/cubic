mod actions;
mod arch;
mod cloudinit;
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
use crate::view::Console;
use clap::Parser;

fn main() {
    let console = &mut view::Stdio::new();
    CommandDispatcher::parse()
        .dispatch(console)
        .map_err(|e| console.error(&e.to_string()))
        .ok();
}
