use crate::view::Console;
use std::io;

#[derive(Debug)]
pub enum Error {
    InvalidArgument(String),
    UnknownArch(String),
    UnknownInstance(String),
    InstanceNotStopped(String),
    InstanceNotRunning(String),
    InstanceAlreadyExists(String),
    Io(io::Error),
    FS(String),
    UnknownImage(String),
    UnsetEnvVar(String),
    CannotParseFile(String),
    CannotShrinkDisk(String),
    CannotOpenTerminal(String),
    HostFwdRuleMalformed(String),
    SystemCommandFailed(String, String),
    Web(reqwest::Error),
    SerdeJson(serde_json::Error),
    SerdeToml(toml::ser::Error),
    InvalidChecksum,
    Ssh(ssh_key::Error),
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
                Error::InstanceNotRunning(name) => format!("Instance '{name}' is not running"),
                Error::InstanceAlreadyExists(id) =>
                    format!("Instance with name '{id}' already exists"),
                Error::Io(e) => format!("{}", e),
                Error::FS(e) => e.to_string(),
                Error::UnknownImage(name) => format!("Unknown image name {name}"),
                Error::UnsetEnvVar(var) => format!("Environment variable '{var}' is not set"),
                Error::CannotParseFile(path) => format!("Cannot parse file '{path}'"),
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
                Error::SerdeToml(err) => format!("[TOML] {err}"),
                Error::Web(e) => format!("{e}"),
                Error::InvalidChecksum => "Verification of image failed".to_string(),
                Error::Ssh(ssh) => format!("SSH error: {ssh}"),
            }
        ))
    }
}
