use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[cfg(test)]
    #[error("Argument error: {0}")]
    InvalidArgument(String),
    #[error("Unknown architecture: '{0}'")]
    UnknownArch(String),
    #[error("Unknown instance '{0}'")]
    UnknownInstance(String),
    #[error("Instance '{0}' is not stopped")]
    InstanceNotStopped(String),
    #[error("Instance '{0}' is not running")]
    InstanceNotRunning(String),
    #[error("Instance with name '{0}' already exists")]
    InstanceAlreadyExists(String),
    #[error("IO Error: {0}")]
    Io(io::Error),
    #[error("FS Error: {0}")]
    FS(String),
    #[error("Unknown image name {0}")]
    UnknownImage(String),
    #[error("Environment variable '{0}' is not set")]
    UnsetEnvVar(String),
    #[error("Cannot parse file '{0}'")]
    CannotParseFile(String),
    #[error("Cannot shrink the disk of the instance '{0}'")]
    CannotShrinkDisk(String),
    #[cfg(not(windows))]
    #[error("Failed to open terminal from path: '{0}'")]
    CannotOpenTerminal(String),
    #[error("System command execution failed\n{0}\n\nReason: {1}")]
    SystemCommandFailed(String, String),
    #[error("Web Error: {0}")]
    Web(reqwest::Error),
    #[error("JSON Error: {0}")]
    #[cfg(not(windows))]
    SerdeJson(serde_json::Error),
    #[error("TOML Error: {0}")]
    SerdeToml(toml::ser::Error),
    #[error("Verification of image failed")]
    InvalidChecksum,
    #[error("Could not detect shell")]
    CouldNotDetectShell,
    #[error("SSH Error: {0}")]
    Ssh(ssh_key::Error),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}
