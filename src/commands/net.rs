mod hostfwd;

use crate::error::Error;
use crate::machine::MachineDao;

use clap::Subcommand;
use hostfwd::HostfwdCommands;

#[derive(Subcommand)]
pub enum NetworkCommands {
    /// Guest to host port forwarding commands
    ///
    /// List forwarded ports of instance "myinstance":
    /// $ cubic net hostfwd list myinstance
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
    pub fn dispatch(&self, machine_dao: &MachineDao) -> Result<(), Error> {
        match self {
            NetworkCommands::Hostfwd(command) => command.dispatch(machine_dao),
        }
    }
}
