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
        if !context.has_instance(&self.instance.name) {
            return Err(Error::NotFound(self.instance.name.clone()));
        }

        if let Some(mut vm) = context.get_vm(&self.instance) {
            if kill {
                vm.quit()?;
            } else {
                vm.stop()?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{ContextBuilder, QMP_REPLIES};
    use crate::models::Environment;
    use crate::platform::SystemMock;
    use std::sync::Arc;

    fn build_instance(name: &str) -> Instance {
        Instance {
            name: name.to_string(),
            ..Instance::default()
        }
    }

    // Kills a running instance and returns what reached its api socket
    fn kill_running_instance() -> String {
        let instance = build_instance("test");
        let builder = ContextBuilder::new(SystemMock::new());
        builder.add_instances(vec![instance.clone()]);
        builder.set_running(&["test"]);
        let system = Arc::clone(builder.get_system());

        StopInstanceAction::new(&instance)
            .run(&builder.build(), true)
            .unwrap();

        system.get_socket_output(Environment::default().get_api_socket("test"))
    }

    #[test]
    fn test_stop_rejects_unknown_instance() {
        let context = ContextBuilder::new(SystemMock::new()).build();

        let result = StopInstanceAction::new(&build_instance("missing")).run(&context, false);

        assert!(matches!(
            result,
            Err(Error::NotFound(ref name)) if name == "missing"
        ));
    }

    #[test]
    fn test_stop_kills_running_instance() {
        assert!(kill_running_instance().contains(r#""method":"quit""#));
    }

    #[test]
    fn test_stop_uses_the_monitor_without_an_api_socket() {
        let env = Environment::default();
        let system = SystemMock::new().add_socket(env.get_monitor_socket("test"), QMP_REPLIES);
        let builder = ContextBuilder::new(system);
        builder.add_instances(vec![build_instance("test")]);
        let system = Arc::clone(builder.get_system());

        let result = StopInstanceAction::new(&build_instance("test")).run(&builder.build(), false);

        assert!(result.is_ok());
        let qmp = system.get_socket_output(env.get_monitor_socket("test"));
        assert!(qmp.ends_with(r#"{"id":"1","execute":"system_powerdown"}"#));
    }

    fn run_with_supervisor_reply(reply: &str) -> (Result<()>, String, String) {
        let env = Environment::default();
        let system = SystemMock::new()
            .add_socket(env.get_api_socket("test"), reply.as_bytes())
            .add_socket(env.get_monitor_socket("test"), QMP_REPLIES);
        let builder = ContextBuilder::new(system);
        builder.add_instances(vec![build_instance("test")]);
        let system = Arc::clone(builder.get_system());

        let result = StopInstanceAction::new(&build_instance("test")).run(&builder.build(), false);

        let api = system.get_socket_output(env.get_api_socket("test"));
        let qmp = system.get_socket_output(env.get_monitor_socket("test"));
        (result, api, qmp)
    }

    #[test]
    fn test_stop_goes_through_the_supervisor() {
        let reply = "{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":null}\n";

        let (result, api, qmp) = run_with_supervisor_reply(reply);

        assert!(result.is_ok());
        assert!(api.contains(r#""method":"stop""#));
        assert!(qmp.is_empty());
    }

    #[test]
    fn test_stop_shows_a_supervisor_error() {
        let reply =
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"error\":{\"code\":-32000,\"message\":\"busy\"}}\n";

        let (result, _, qmp) = run_with_supervisor_reply(reply);

        assert!(matches!(result, Err(Error::VmRequestFailed(message)) if message == "busy"));
        assert!(qmp.is_empty());
    }
}
