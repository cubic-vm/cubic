use crate::commands::Context;
use crate::models::InstanceName;
use std::str::FromStr;

/// Lists the names of all instances, sorted without regard to case.
#[derive(Default)]
pub struct ListInstancesAction;

impl ListInstancesAction {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self, context: &Context) -> Vec<String> {
        let dir = context.get_env().get_instance_dir();
        let mut names: Vec<String> = context
            .get_system()
            .read_dir(&dir)
            .unwrap_or_default()
            .iter()
            .filter_map(|entry| entry.file_name())
            .filter_map(|name| name.to_str())
            .filter(|name| InstanceName::from_str(name).is_ok())
            .map(|name| name.to_string())
            .collect();
        names.sort_by_key(|name| name.to_lowercase());
        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::ContextBuilder;
    use crate::models::Environment;
    use crate::platform::SystemMock;

    #[test]
    fn test_run_lists_sorted_valid_names() {
        let env = Environment::default();
        let system = SystemMock::new()
            .add_dir(env.get_instance_dir2("zebra"))
            .add_dir(env.get_instance_dir2("apple"))
            .add_dir(env.get_instance_dir2("web.tmp"));
        let context = ContextBuilder::new(system).build();

        assert_eq!(ListInstancesAction::new().run(&context), ["apple", "zebra"]);
    }
}
