use crate::commands::Context;
use crate::error::Result;
use crate::instance::InstanceSerializer;
use crate::models::Instance;
use crate::platform::Access;

/// Writes the config of an instance through a temporary file, so a crash
/// never leaves half of it behind.
#[derive(Default)]
pub struct StoreInstanceAction;

impl StoreInstanceAction {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self, context: &Context, instance: &Instance) -> Result<()> {
        let system = context.get_system();
        let env = context.get_env();
        let path = env.get_instance_toml_config_file(&instance.name);
        let temp_path = path.with_added_extension("tmp");
        let content = InstanceSerializer::new().serialize(instance)?;
        system.write_file(&temp_path, content.as_bytes(), Access::Normal)?;
        system.rename_file(&temp_path, &path)
    }
}
