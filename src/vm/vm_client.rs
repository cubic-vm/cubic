use crate::error::{Error, Result};
use crate::models::{Environment, Instance};
use crate::platform::System;

pub struct VmClient;

impl VmClient {
    pub fn connect(system: &dyn System, env: &Environment, instance: &Instance) -> Result<Self> {
        let socket = env.get_api_socket(&instance.name);
        system
            .connect_socket(&socket, None)
            .map_err(|_| Error::InstanceNotRunning(instance.name.clone()))?;
        Ok(VmClient)
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
