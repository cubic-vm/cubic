use crate::commands;
use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore, PortForward};
use crate::view::Console;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum HostfwdCommands {
    /// List forwarded host ports (Deprecated)
    ///
    /// List forwarded ports for all instances:
    /// $ cubic net hostfwd list
    #[clap(verbatim_doc_comment, alias = "ls")]
    List(commands::ListPortCommand),

    /// Add host port forwarding rule (Deprecated)
    ///
    /// Forward guest SSH port (TCP port 22) to host on port 8000:
    /// $ cubic net hostfwd add myinstance tcp:127.0.0.1:8000-:22
    #[clap(verbatim_doc_comment)]
    Add {
        /// Virtual machine instance
        instance: String,
        /// Port forwarding rule
        rule: String,
    },

    /// Delete host port forwarding rule (Deprecated)
    ///
    /// Remove port forwarding:
    /// $ cubic net hostfwd del myinstance tcp:127.0.0.1:8000-:22
    #[clap(verbatim_doc_comment, alias = "rm")]
    Del {
        /// Virtual machine instance
        instance: String,
        /// Port forwarding rule
        rule: String,
    },
}

impl HostfwdCommands {
    pub fn dispatch(
        &self,
        console: &mut dyn Console,
        instance_dao: &InstanceDao,
    ) -> Result<(), Error> {
        match self {
            HostfwdCommands::List(cmd) => cmd.run(console, instance_dao),
            HostfwdCommands::Add { instance, rule } => match PortForward::from_qemu(rule) {
                Ok(rule) => {
                    let mut instance = instance_dao.load(instance)?;
                    instance.hostfwd.push(rule);
                    instance_dao.store(&instance)
                }
                Err(msg) => Err(Error::HostFwdRuleMalformed(msg)),
            },
            HostfwdCommands::Del { instance, rule } => match PortForward::from_qemu(rule) {
                Ok(rule) => {
                    let mut instance = instance_dao.load(instance)?;
                    instance.hostfwd.retain(|item| item != &rule);
                    instance_dao.store(&instance)
                }
                Err(msg) => Err(Error::HostFwdRuleMalformed(msg)),
            },
        }
    }
}
