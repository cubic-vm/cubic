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

        if !kill {
            match context.get_vm(&self.instance).and_then(|mut vm| vm.stop()) {
                Ok(()) => return Ok(()),
                // A live supervisor already tried QEMU, so its error is final
                Err(e @ Error::VmRequestFailed(_)) => return Err(e),
                // No supervisor or no answer, as with instances of older versions
                Err(_) => {}
            }
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

    // Runs the action on a running instance and returns what reached its monitor
    fn run_on_running_instance(kill: bool) -> String {
        let instance = build_instance("test");
        let builder = ContextBuilder::new(SystemMock::new());
        builder.add_instances(vec![instance.clone()]);
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
            Err(Error::NotFound(ref name)) if name == "missing"
        ));
    }

    #[test]
    fn test_stop_kills_running_instance() {
        assert!(run_on_running_instance(true).ends_with(r#"{"id":"1","execute":"quit"}"#));
    }

    #[test]
    fn test_stop_falls_back_to_the_monitor_without_a_supervisor_reply() {
        let output = run_on_running_instance(false);

        assert!(output.ends_with(r#"{"id":"1","execute":"system_powerdown"}"#));
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
    fn test_stop_shows_a_supervisor_error_without_falling_back() {
        let reply =
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"error\":{\"code\":-32000,\"message\":\"busy\"}}\n";

        let (result, _, qmp) = run_with_supervisor_reply(reply);

        assert!(matches!(result, Err(Error::VmRequestFailed(message)) if message == "busy"));
        assert!(qmp.is_empty());
    }
}
