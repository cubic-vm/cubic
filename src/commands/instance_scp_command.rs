use crate::commands::Verbosity;
use crate::error::Error;
use crate::instance::{InstanceStore, TargetPath};
use crate::ssh_cmd::{get_ssh_private_key_names, Scp};
use clap::Parser;
use std::env;

/// Copy a file from or to a virtual machine instance with SCP
#[derive(Parser)]
pub struct InstanceScpCommand {
    /// Source of the data to copy
    from: TargetPath,
    /// Target of the data to copy
    to: TargetPath,
    /// Enable verbose logging
    #[clap(short, long, default_value_t = false)]
    verbose: bool,
    /// Reduce logging output
    #[clap(short, long, default_value_t = false)]
    quiet: bool,
    /// Pass additional SCP arguments
    #[clap(long)]
    scp_args: Option<String>,
}

impl InstanceScpCommand {
    pub fn run(&self, instance_store: &dyn InstanceStore) -> Result<(), Error> {
        let from = &self.from.to_scp(instance_store)?;
        let to = &self.to.to_scp(instance_store)?;
        let verbosity = Verbosity::new(self.verbose, self.quiet);

        Scp::new()
            .set_root_dir(env::var("SNAP").unwrap_or_default().as_str())
            .set_verbose(verbosity.is_verbose())
            .set_known_hosts_file(
                env::var("HOME")
                    .map(|dir| format!("{dir}/.ssh/known_hosts"))
                    .ok(),
            )
            .set_private_keys(get_ssh_private_key_names()?)
            .set_args(self.scp_args.as_ref().unwrap_or(&String::new()))
            .copy(from, to)
            .set_stdout(!verbosity.is_quiet())
            .run()
    }
}
