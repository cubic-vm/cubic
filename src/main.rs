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
    let console = &mut view::Stdio::new();
    match CommandDispatcher::parse().dispatch(console) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            console.error(&e.to_string());
            ExitCode::FAILURE
        }
    }
}
