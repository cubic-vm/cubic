use crate::error::Error;
use crate::machine::MachineDao;
use crate::view::{Alignment, TableView};
use clap::Subcommand;
use regex::Regex;

#[derive(Subcommand)]
pub enum HostfwdCommands {
    /// List forwarded host ports
    ///
    /// List forwarded ports for all instances:
    /// $ cubic net hostfwd list
    #[clap(verbatim_doc_comment)]
    List,

    /// Add host port forwarding rule
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

    /// Delete host port forwarding rule
    ///
    /// Remove port forwarding:
    /// $ cubic net hostfwd del myinstance tcp:127.0.0.1:8000-:22
    #[clap(verbatim_doc_comment)]
    Del {
        /// Virtual machine instance
        instance: String,
        /// Port forwarding rule
        rule: String,
    },
}

impl HostfwdCommands {
    pub fn dispatch(&self, machine_dao: &MachineDao) -> Result<(), Error> {
        match self {
            HostfwdCommands::List => {
                let machine_names = machine_dao.get_machines();

                let mut view = TableView::new();
                view.add_row()
                    .add("INSTANCE", Alignment::Left)
                    .add("RULE", Alignment::Left);

                for machine_name in machine_names {
                    for hostfwd in machine_dao.load(&machine_name)?.hostfwd {
                        view.add_row()
                            .add(&machine_name, Alignment::Left)
                            .add(&hostfwd, Alignment::Left);
                    }
                }
                view.print();
                Ok(())
            }
            HostfwdCommands::Add { instance, rule } => {
                if !Regex::new(
                    r"^(udp|tcp):([0-9]+\.){3}[0-9]+:[0-9]{1,5}\-([0-9]+.[0-9])?:[0-9]{1,5}$",
                )
                .unwrap()
                .is_match(rule)
                {
                    return Err(Error::HostFwdRuleMalformed(rule.to_string()));
                }
                let mut machine = machine_dao.load(instance)?;
                machine.hostfwd.push(rule.to_string());
                machine_dao.store(&machine)
            }
            HostfwdCommands::Del { instance, rule } => {
                let mut machine = machine_dao.load(instance)?;
                machine.hostfwd.retain(|item| item != rule);
                machine_dao.store(&machine)
            }
        }
    }
}
