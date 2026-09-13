mod actions;
mod cloudinit;
mod commands;
mod env;
mod error;
mod image;
mod instance;
mod iso9660;
mod models;
mod platform;
mod qemu;
mod ssh;
mod util;
mod view;
mod web;

use crate::commands::CommandDispatcher;
use crate::platform::{OsSystem, System};
use clap::Parser;
use std::sync::Arc;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ! {
    // Disable raw mode before the default panic hook runs, so a panic during
    // an interactive session (ssh, console) does not leave the terminal
    // broken. This also covers panic = 'abort' builds, since the hook runs
    // before the process aborts.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        crossterm::terminal::disable_raw_mode().ok();
        default_hook(info);
    }));

    // reqwest picks up the process wide crypto provider, and rustls is
    // built with ring
    rustls::crypto::ring::default_provider()
        .install_default()
        .ok();

    let system: Arc<dyn System> = Arc::new(OsSystem::new());
    let console = &view::Console::new(Arc::clone(&system));
    let result = CommandDispatcher::parse()
        .dispatch(Arc::clone(&system), console)
        .await;
    if let Err(error) = &result {
        console.error(&error.to_string());
    }
    let code = result.map_or(1, i32::from);

    console.flush();
    system.exit(code);
}
