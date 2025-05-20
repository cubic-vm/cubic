use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore};
use crate::view::{Alignment, Stdio, TableView};
use clap::Subcommand;
use regex::Regex;

#[derive(Subcommand)]
pub enum HostfwdCommands {
    /// List forwarded host ports
    ///
    /// List forwarded ports for all instances:
    /// $ cubic net hostfwd list
    #[clap(verbatim_doc_comment, alias = "ls")]
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
    #[clap(verbatim_doc_comment, alias = "rm")]
    Del {
        /// Virtual machine instance
        instance: String,
        /// Port forwarding rule
        rule: String,
    },
}

impl HostfwdCommands {
    pub fn dispatch(&self, instance_dao: &InstanceDao) -> Result<(), Error> {
        let console = &mut Stdio::new();
        match self {
            HostfwdCommands::List => {
                let instance_names = instance_dao.get_instances();

                let mut view = TableView::new();
                view.add_row()
                    .add("INSTANCE", Alignment::Left)
                    .add("RULE", Alignment::Left);

                for instance_name in instance_names {
                    for hostfwd in instance_dao.load(&instance_name)?.hostfwd {
                        view.add_row()
                            .add(&instance_name, Alignment::Left)
                            .add(&hostfwd, Alignment::Left);
                    }
                }
                view.print(console);
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
                let mut instance = instance_dao.load(instance)?;
                instance.hostfwd.push(rule.to_string());
                instance_dao.store(&instance)
            }
            HostfwdCommands::Del { instance, rule } => {
                let mut instance = instance_dao.load(instance)?;
                instance.hostfwd.retain(|item| item != rule);
                instance_dao.store(&instance)
            }
        }
    }
}
