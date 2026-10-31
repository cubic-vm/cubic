use crate::commands::Context;
use crate::error::{Error, Result};
use crate::models::Instance;

pub struct StopInstanceAction {
    instance: Instance,
}

impl StopInstanceAction {
    pub fn new(instance: &Instance) -> Self {
        Self {
            instance: instance.clone(),
        }
    }

    pub fn run(&mut self, context: &Context, kill: bool) -> Result<()> {
        if !context.get_instance_store().exists(&self.instance.name) {
            return Err(Error::UnknownInstance(self.instance.name.clone()));
        }

        let mut monitor = context.get_monitor(&self.instance);
        if monitor.is_running() {
            if kill {
                monitor.kill()?;
            } else {
                monitor.shutdown()?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::ContextBuilder;
    use crate::models::Environment;
    use crate::platform::SystemMock;
    use std::sync::Arc;

    fn build_instance(name: &str) -> Instance {
        Instance {
            name: name.to_string(),
            ..Instance::default()
        }
    }

    // Runs the action on a running instance and returns what reached its monitor
    fn run_on_running_instance(kill: bool) -> String {
        let instance = build_instance("test");
        let mut builder = ContextBuilder::new(SystemMock::new());
        builder.get_store().add_instances(vec![instance.clone()]);
        builder.set_running(&["test"]);
        let system = Arc::clone(builder.get_system());

        StopInstanceAction::new(&instance)
            .run(&builder.build(), kill)
            .unwrap();

        system.get_socket_output(Environment::default().get_monitor_socket("test"))
    }

    #[test]
    fn test_stop_rejects_unknown_instance() {
        let context = ContextBuilder::new(SystemMock::new()).build();

        let result = StopInstanceAction::new(&build_instance("missing")).run(&context, false);

        assert!(matches!(
            result,
            Err(Error::UnknownInstance(ref name)) if name == "missing"
        ));
    }

    #[test]
    fn test_stop_kills_running_instance() {
        assert!(run_on_running_instance(true).ends_with(r#"{"id":"1","execute":"quit"}"#));
    }

    #[test]
    fn test_stop_without_kill_requests_monitor_shutdown() {
        let output = run_on_running_instance(false);

        assert!(output.ends_with(r#"{"id":"1","execute":"system_powerdown"}"#));
    }
}
