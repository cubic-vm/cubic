use crate::actions::{DeleteInstanceAction, LoadInstanceAction};
use crate::commands::Context;
use crate::error::{Error, Result};

/// Frees a name that only a stale instance still holds
#[derive(Default)]
pub struct ClaimInstanceNameAction;

impl ClaimInstanceNameAction {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self, context: &Context, name: &str) -> Result<()> {
        match LoadInstanceAction::new().read(context, name) {
            Err(Error::NotFound(_)) => Ok(()),
            Ok(instance) if context.is_stale(&instance) => {
                DeleteInstanceAction::new().run(context, &instance)
            }
            _ => Err(Error::InstanceAlreadyExists(name.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::ContextBuilder;
    use crate::models::{Environment, Instance};
    use crate::platform::{FileSystem, SystemMock};
    use std::sync::Arc;

    fn build_builder() -> ContextBuilder {
        let builder = ContextBuilder::new(SystemMock::new());
        builder.add_instances(vec![Instance {
            name: "test".to_string(),
            auto_remove: true,
            ..Instance::default()
        }]);
        builder
    }

    #[test]
    fn test_claim_name_accepts_a_free_name() {
        let context = build_builder().build();
        let action = ClaimInstanceNameAction::new();

        assert!(action.run(&context, "free").is_ok());
    }

    #[test]
    fn test_claim_name_deletes_a_stale_instance() {
        let builder = build_builder();
        let system = Arc::clone(builder.get_system());

        let context = builder.build();
        let action = ClaimInstanceNameAction::new();
        action.run(&context, "test").unwrap();

        let dir = Environment::default().get_instance_dir2("test");
        assert!(!system.exists_path(&dir));
    }

    #[test]
    fn test_claim_name_rejects_a_running_auto_remove_instance() {
        let builder = build_builder();
        builder.set_running(&["test"]);

        let context = builder.build();
        let result = ClaimInstanceNameAction::new().run(&context, "test");

        assert!(matches!(
            result,
            Err(Error::InstanceAlreadyExists(ref name)) if name == "test"
        ));
    }
}
