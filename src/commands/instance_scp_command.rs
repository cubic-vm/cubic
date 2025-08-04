use crate::commands::Verbosity;
use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore};
use crate::ssh_cmd::{get_ssh_private_key_names, Scp};
use clap::Parser;
use std::env;

fn get_scp_address(instance_dao: &InstanceDao, location: &str) -> Result<String, Error> {
    Ok(if location.contains(':') {
        let mut location_token = location.split(':');
        let name = location_token.next().unwrap();
        let path = location_token.next().unwrap();
        let instance = instance_dao.load(name)?;
        let port = instance.ssh_port;
        let user = instance.user;
        format!("scp://{user}@127.0.0.1:{port}/{path}")
    } else {
        location.to_string()
    })
}

/// Copy a file from or to a virtual machine instance with SCP
#[derive(Parser)]
pub struct InstanceScpCommand {
    /// Source of the data to copy
    from: String,
    /// Target of the data to copy
    to: String,
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
    pub fn run(&self, instance_dao: &InstanceDao) -> Result<(), Error> {
        let from = &get_scp_address(instance_dao, &self.from)?;
        let to = &get_scp_address(instance_dao, &self.to)?;
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
