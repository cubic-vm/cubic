mod commands;
mod emulator;
mod error;
mod image;
mod instance;
mod qemu;
mod ssh_cmd;
mod util;
mod view;

use crate::commands::{dispatch, CommandDispatcher};
use clap::Parser;

fn main() {
    util::migrate();

    CommandDispatcher::parse()
        .command
        .ok_or(error::Error::UnknownCommand)
        .and_then(dispatch)
        .map_err(error::print_error)
        .ok();
}
