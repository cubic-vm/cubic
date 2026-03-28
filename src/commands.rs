mod clone_command;
mod command_dispatcher;
mod completions_command;
mod console_command;
mod create_command;
mod delete_command;
mod exec_command;
mod image;
mod list_image_command;
mod list_instance_command;
mod list_port_command;
mod modify_command;
mod prune_command;
mod rename_command;
mod restart_command;
mod run_command;
mod scp_command;
mod show_command;
mod show_image_command;
mod show_instance_command;
mod ssh_command;
mod start_command;
mod stop_command;
mod verbosity;

pub use clone_command::*;
pub use command_dispatcher::*;
pub use completions_command::*;
pub use console_command::*;
pub use create_command::*;
pub use delete_command::*;
pub use exec_command::*;
pub use image::*;
pub use list_image_command::*;
pub use list_instance_command::*;
pub use list_port_command::*;
pub use modify_command::*;
pub use prune_command::*;
pub use rename_command::*;
pub use restart_command::*;
pub use run_command::*;
pub use scp_command::*;
pub use show_command::*;
pub use show_image_command::*;
pub use show_instance_command::*;
pub use ssh_command::*;
pub use start_command::*;
pub use stop_command::*;
pub use verbosity::*;

use crate::env::Environment;
use crate::error::Result;
use crate::image::ImageStore;
use crate::instance::InstanceStore;
use crate::view::Console;

trait Command {
    fn run(
        &self,
        console: &mut dyn Console,
        env: &Environment,
        image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<()>;
}
