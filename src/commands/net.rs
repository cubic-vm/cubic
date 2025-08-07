mod hostfwd;

use crate::error::Error;
use crate::instance::InstanceDao;

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

impl NetworkCommands {
    pub fn dispatch(&self, instance_dao: &InstanceDao) -> Result<(), Error> {
        match self {
            NetworkCommands::Hostfwd(command) => command.dispatch(instance_dao),
        }
    }
}
