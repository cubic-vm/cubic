mod cli;
mod commands;
mod error;
mod image;
mod machine;
mod util;

use clap::Parser;

fn main() {
    util::migrate();

    cli::Cli::parse()
        .command
        .ok_or(error::Error::UnknownCommand)
        .and_then(cli::dispatch)
        .map_err(error::print_error)
        .ok();
}
