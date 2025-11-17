mod hostfwd;

use crate::commands::Command;
use crate::env::Environment;
use crate::error::Error;
use crate::image::ImageStore;
use crate::instance::InstanceStore;
use crate::view::Console;
use clap::Subcommand;
use hostfwd::HostfwdCommands;

#[derive(Subcommand)]
pub enum NetworkCommands {
    /// Guest to host port forwarding commands (Deprecated)
    ///
    /// List forwarded ports for all instances:
    /// $ cubic net hostfwd list
    ///
    /// Forward guest SSH port (TCP port 22) to host on port 8000:
    /// $ cubic net hostfwd add myinstance tcp:127.0.0.1:8000-:22
    ///
    /// Remove port forwarding:
    /// $ cubic net hostfwd del myinstance tcp:127.0.0.1:8000-:22
    #[command(subcommand, verbatim_doc_comment)]
    Hostfwd(HostfwdCommands),
}

impl Command for NetworkCommands {
    fn run(
        &self,
        console: &mut dyn Console,
        env: &Environment,
        image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<(), Error> {
        match self {
            NetworkCommands::Hostfwd(cmd) => cmd.run(console, env, image_store, instance_store),
        }
    }
}
