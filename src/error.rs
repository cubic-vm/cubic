use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(
        "CPU arch '{0}' is not support.\n\nChoose a supported architecture: 'amd64' or 'arm64'"
    )]
    UnknownArch(String),

    #[error(
        "Instance '{0}' does not exist.\n\nOptions:\n  - Use an existing instance name\n  - Create it first: `cubic create {0} [...]`"
    )]
    UnknownInstance(String),

    #[error("Image '{0}' not found.\n\nList available images with: `cubic images`")]
    UnknownImage(String),

    #[error(
        "Instance '{0}' must be stopped to proceed.\n\nRun `cubic stop --wait {0}` to stop it now."
    )]
    InstanceNotStopped(String),

    #[error("Instance '{0}' is not running")]
    InstanceNotRunning(String),

    #[error(
        "Instance name '{0}' is already taken.\n\nOptions:\n  - Choose a different name\n  - Connect to existing instance: `cubic ssh {0}`"
    )]
    InstanceAlreadyExists(String),

    #[error(
        "Environment variable '{0}' is not set.\n\nTemporary (current session):\n  - Linux/macOS: export {0}=value\n  - Windows (PowerShell): $env:{0} = \"value\"\n  - Windows (CMD): set {0}=value\n\nPermanent: Add to your shell profile or Windows Environment Variables settings."
    )]
    UnsetEnvVar(String),

    #[error(
        "Failed to execute a system command.

Command:

{0}

Error:

{1}

Troubleshoot:
  - Make sure the following tools are installed and the PATH variable is set
    correctly:
    - qemu-system-x86_64
    - qemu-system-aarch64
    - qemu-img

  - Report error at https://github.com/cubic-vm/cubic/issues
"
    )]
    SystemCommandFailed(String, String),

    #[error("IO Error: {0}")]
    Io(#[from] io::Error),

    #[error("FS Error: {0}")]
    FS(String),

    #[error("Cannot shrink the disk of the instance '{0}'")]
    CannotShrinkDisk(String),

    #[cfg(not(windows))]
    #[error("Failed to open terminal from path: '{0}'")]
    CannotOpenTerminal(String),

    #[error("Web Error: {0}")]
    Web(#[from] reqwest::Error),

    #[error("JSON Error: {0}")]
    #[cfg(not(windows))]
    SerdeJson(#[from] serde_json::Error),

    #[error("TOML Error: {0}")]
    SerdeToml(#[from] toml::ser::Error),

    #[error("Verification of image failed")]
    InvalidChecksum,

    #[error("Could not detect shell")]
    CouldNotDetectShell,

    #[error("SSH Error: {0}")]
    Ssh(#[from] ssh_key::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("SCP Error: {0}")]
    Scp(String),
}
