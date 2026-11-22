use crate::error::{Error, Result};
use crate::models::{Environment, Instance};
use crate::platform::{Socket, System};
use crate::vm::VmControl;
use crate::vm::vm_message::VmMessage;
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
}

impl VmControl for VmClient {
    fn stop(&mut self) -> Result<()> {
        let mut request = serde_json::to_string(&VmMessage::build_request("stop"))?;
        request.push('\n');
        let socket = self.stream.get_mut();
        socket.write_all(request.as_bytes())?;
        socket.flush()?;

        let mut line = String::new();
        self.stream.read_line(&mut line)?;
        match serde_json::from_str(&line)? {
            VmMessage::Result { .. } => Ok(()),
            VmMessage::Error { error, .. } => Err(Error::VmRequestFailed(error.message)),
            VmMessage::Request { .. } => Err(Error::VmRequestFailed("Invalid reply".to_string())),
        }
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
    fn test_connect_follows_the_api_socket() {
        let env = Environment::default();
        let system = SystemMock::new().add_socket(env.get_api_socket("test"), b"");

        assert!(VmClient::connect(&system, &env, &build_instance()).is_ok());
    }
}
