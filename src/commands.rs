pub mod clone_command;
pub mod command_dispatcher;
pub mod completions_command;
pub mod console_command;
pub mod create_command;
pub mod delete_command;
pub mod exec_command;
pub mod image;
pub mod list_image_command;
pub mod list_instance_command;
pub mod list_port_command;
pub mod modify_command;
pub mod prune_command;
pub mod rename_command;
pub mod restart_command;
pub mod run_command;
pub mod scp_command;
pub mod show_command;
pub mod show_image_command;
pub mod show_instance_command;
pub mod ssh_command;
pub mod start_command;
pub mod stop_command;
pub mod verbosity;

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
