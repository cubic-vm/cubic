use crate::error::{Error, Result};
use crate::models::{Environment, Instance};
use crate::platform::{Socket, System};
use crate::vm::vm_message::{VmMessage, VmMethod};
use crate::vm::{VmControl, VmState};
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::time::Duration;

const API_TIMEOUT: Duration = Duration::from_secs(5);

pub struct VmClient {
    stream: BufReader<Box<dyn Socket>>,
}

impl VmClient {
    pub fn connect(system: &dyn System, env: &Environment, instance: &Instance) -> Result<Self> {
        let socket = env.get_api_socket(&instance.name);
        let socket = system
            .connect_socket(&socket, Some(API_TIMEOUT))
            .map_err(|_| Error::InstanceNotRunning(instance.name.clone()))?;
        Ok(VmClient {
            stream: BufReader::new(socket),
        })
    }

    fn call(&mut self, method: VmMethod) -> Result<Value> {
        let mut request = serde_json::to_string(&VmMessage::build_request(method))?;
        request.push('\n');
        let socket = self.stream.get_mut();
        socket.write_all(request.as_bytes())?;
        socket.flush()?;

        let mut line = String::new();
        self.stream.read_line(&mut line)?;
        match serde_json::from_str(&line)? {
            VmMessage::Result { result, .. } => Ok(result),
            VmMessage::Error { error, .. } => Err(Error::VmRequestFailed(error.message)),
            VmMessage::Request { .. } => Err(Error::VmRequestFailed("Invalid reply".to_string())),
        }
    }
}

impl VmControl for VmClient {
    fn stop(&mut self) -> Result<()> {
        self.call(VmMethod::Stop).map(|_| ())
    }

    fn quit(&mut self) -> Result<()> {
        self.call(VmMethod::Quit).map(|_| ())
    }

    fn get_state(&mut self) -> Result<VmState> {
        Ok(serde_json::from_value(self.call(VmMethod::GetState)?)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::SystemMock;

    fn build_instance() -> Instance {
        Instance {
            name: "test".to_string(),
            ..Instance::default()
        }
    }

    #[test]
    fn test_connect_fails_when_nothing_answers() {
        let env = Environment::default();

        let result = VmClient::connect(&SystemMock::new(), &env, &build_instance());

        assert!(matches!(result, Err(Error::InstanceNotRunning(name)) if name == "test"));
    }

    #[test]
    fn test_get_state_reads_the_state_from_the_reply() {
        let env = Environment::default();
        let reply = b"{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":\"stopping\"}\n";
        let system = SystemMock::new().add_socket(env.get_api_socket("test"), reply);
        let mut client = VmClient::connect(&system, &env, &build_instance()).unwrap();

        assert_eq!(client.get_state().unwrap(), VmState::Stopping);
    }
}
