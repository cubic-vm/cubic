use crate::error::{Error, Result};
use crate::models::{Environment, Instance};
use crate::platform::{Listener, System};
use std::path::PathBuf;

pub struct VmServer<'a> {
    system: &'a dyn System,
    path: PathBuf,
    listener: Box<dyn Listener>,
}

impl<'a> VmServer<'a> {
    pub fn bind(system: &'a dyn System, env: &Environment, instance: &Instance) -> Result<Self> {
        let path = env.get_api_socket(&instance.name);
        if system.connect_socket(&path, None).is_ok() {
            return Err(Error::InstanceNotStopped(instance.name.clone()));
        }
        system.remove_file(&path).ok();
        let listener = system.bind_socket(&path)?;
        Ok(VmServer {
            system,
            path,
            listener,
        })
    }

    pub fn accept_client(&self) -> bool {
        self.listener.accept().is_ok()
    }
}

impl Drop for VmServer<'_> {
    fn drop(&mut self) {
        self.system.remove_file(&self.path).ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::{FileSystem, SystemMock};
    use crate::vm::VmClient;

    fn build_instance() -> Instance {
        Instance {
            name: "test".to_string(),
            ..Instance::default()
        }
    }

    #[test]
    fn test_bind_fails_while_a_vm_answers() {
        let env = Environment::default();
        let system = SystemMock::new().add_socket(env.get_api_socket("test"), b"");

        let result = VmServer::bind(&system, &env, &build_instance());

        assert!(matches!(result, Err(Error::InstanceNotStopped(name)) if name == "test"));
    }

    #[test]
    fn test_bind_replaces_a_leftover_socket_file() {
        let env = Environment::default();
        let path = env.get_api_socket("test");
        let system = SystemMock::new().add_file(&path, b"");

        let _server = VmServer::bind(&system, &env, &build_instance()).unwrap();

        assert!(!system.exists_path(&path));
        assert!(VmClient::connect(&system, &env, &build_instance()).is_ok());
    }
}
