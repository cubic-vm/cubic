mod commands;
mod emulator;
mod error;
mod image;
mod instance;
mod qemu;
mod ssh_cmd;
mod util;
mod view;

use crate::commands::CommandDispatcher;

fn main() {
    util::migrate();

    CommandDispatcher::new()
        .dispatch()
        .map_err(error::print_error)
        .ok();
}
