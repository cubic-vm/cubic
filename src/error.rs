use crate::view::Console;
use std::io;

#[derive(Debug)]
pub enum Error {
    InvalidArgument(String),
    UnknownArch(String),
    UnknownInstance(String),
    InstanceNotStopped(String),
    InstanceAlreadyExists(String),
    Io(io::Error),
    FS(String),
    UnknownImage(String),
    UnsetEnvVar(String),
    CannotParseFile(String),
    CannotParseSize(String),
    CannotShrinkDisk(String),
    CannotOpenTerminal(String),
    HostFwdRuleMalformed(String),
    SystemCommandFailed(String, String),
    Web(reqwest::Error),
    SerdeJson(serde_json::Error),
    SerdeYaml(serde_yaml::Error),
}

impl Error {
    pub fn print(&self, console: &mut dyn Console) {
        console.error(&format!(
            "ERROR: {}",
            match self {
                Error::InvalidArgument(err) => format!("Argument error: {err}"),
                Error::UnknownArch(name) => format!("Unknown architecture: '{name}'"),
                Error::UnknownInstance(instance) => format!("Unknown instance '{instance}'"),
                Error::InstanceNotStopped(name) => format!("Instance '{name}' is not stopped"),
                Error::InstanceAlreadyExists(id) =>
                    format!("Instance with name '{id}' already exists"),
                Error::Io(e) => format!("{}", e),
                Error::FS(e) => e.to_string(),
                Error::UnknownImage(name) => format!("Unknown image name {name}"),
                Error::UnsetEnvVar(var) => format!("Environment variable '{var}' is not set"),
                Error::CannotParseFile(path) => format!("Cannot parse file '{path}'"),
                Error::CannotParseSize(size) => format!("Invalid data size format '{size}'"),
                Error::CannotShrinkDisk(name) => {
                    format!("Cannot shrink the disk of the instance '{name}'")
                }
                Error::CannotOpenTerminal(path) =>
                    format!("Failed to open terminal from path: '{path}'"),
                Error::HostFwdRuleMalformed(rule) =>
                    format!("Host forwarding rule is malformed: {rule}"),
                Error::SystemCommandFailed(cmd, stderr) => {
                    format!(
                        "System command execution failed\n{cmd}\n\nReason: {}",
                        stderr.trim()
                    )
                }
                Error::SerdeJson(err) => format!("[JSON] {err}"),
                Error::SerdeYaml(err) => format!("[YAML] {err}"),
                Error::Web(e) => format!("{e}"),
            }
        ))
    }
}
