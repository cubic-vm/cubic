use crate::commands::Context;
use crate::error::Result;
use crate::models::Instance;

/// Removes the directory of an instance with its config and disk.
#[derive(Default)]
pub struct DeleteInstanceAction;

impl DeleteInstanceAction {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self, context: &Context, instance: &Instance) -> Result<()> {
        let dir = context.get_env().get_instance_dir2(&instance.name);
        context.get_system().remove_dir(&dir)
    }
}
