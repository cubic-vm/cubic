use crate::models::Arch;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Names the operation so every file system implementation words it the same.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FsOperation {
    CreateDir,
    ReadMetadata,
    RemoveDir,
    ReadDir,
    CreateFile,
    OpenFile,
    ReadFile,
    WriteFile,
    RemoveFile,
}

impl fmt::Display for FsOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FsOperation::CreateDir => "create directory",
                FsOperation::ReadMetadata => "read metadata of",
                FsOperation::RemoveDir => "remove directory",
                FsOperation::ReadDir => "read directory",
                FsOperation::CreateFile => "create file",
                FsOperation::OpenFile => "open file",
                FsOperation::ReadFile => "read file",
                FsOperation::WriteFile => "write file",
                FsOperation::RemoveFile => "delete file",
            }
        )
    }
}

#[derive(Error, Debug)]
pub enum Error {
    // Instances
    #[error(
        "Instance '{0}' does not exist.\n\nOptions:\n  - Use an existing instance name\n  - Create it first: `cubic create {0} [...]`"
    )]
    UnknownInstance(String),

    #[error("Config of instance '{name}' is invalid.\n\n{source}")]
    InvalidInstanceConfig {
        name: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("No instance name was given.\n\nProvide at least one instance name.")]
    MissingInstanceName,

    #[error(
        "Instance name '{0}' is already taken.\n\nOptions:\n  - Choose a different name\n  - Connect to existing instance: `cubic ssh {0}`"
    )]
    InstanceAlreadyExists(String),

    #[error(
        "Instance '{0}' must be stopped to proceed.\n\nRun `cubic stop --wait {0}` to stop it now."
    )]
    InstanceNotStopped(String),

    #[error("Instance '{0}' is not running")]
    InstanceNotRunning(String),

    #[error(
        "Timed out waiting for instance(s) to start.\n\nTroubleshoot:\n  - Run with --verbose to see the QEMU command\n  - Check that QEMU can open /dev/kvm and firmware files\n  - Try again; the system may be under load\n"
    )]
    StartTimeout,

    #[error(
        "Timed out waiting for the console of instance '{0}'.\n\nTroubleshoot:\n  - Check that the instance is running: `cubic list`\n  - Run with --verbose to see the QEMU command\n  - Try again; the system may be under load\n"
    )]
    ConsoleTimeout(String),

    #[error(
        "Not enough free memory to start instance '{0}'.\n\nTroubleshoot:\n  - Free up memory by stopping other instances or processes\n  - Reduce the instance memory: `cubic modify {0} --memory <size>`\n  - Accept the proposed smaller size by running with --yes\n"
    )]
    NotEnoughMemory(String),

    #[error("Cannot shrink the disk of the instance '{0}'")]
    CannotShrinkDisk(String),

    #[error(
        "Hardware acceleration needs a guest arch equal to the host arch.\n\nInstance '{0}' is {1} and this host is {2}.\n\nRun it with `--accel off` to use software emulation."
    )]
    ArchMismatch(String, Arch, Arch),

    // Images
    #[error("Image '{0}' not found.\n\nList available images with: `cubic images`")]
    UnknownImage(String),

    #[error("Verification of image failed")]
    InvalidChecksum,

    // QEMU and system commands
    #[error("{}", format_qemu_not_found_help())]
    QemuNotFound,

    #[error("System command '{0}' was not found on PATH")]
    SystemCommandNotFound(String),

    #[error(
        "Failed to execute a system command.

Command:

{0}

Error:

{1}

Troubleshoot:
  - Run the command again with --verbose to see the full QEMU command line
  - Check the QEMU version with `qemu-system-x86_64 --version`

  - Report error at https://github.com/cubic-vm/cubic/issues
"
    )]
    SystemCommandFailed(String, String),

    #[error("Failed to apply port forwarding rule on the running instance: {0}")]
    HostfwdCommandFailed(String),

    #[error("Process {0} is not running")]
    ProcessNotFound(u64),

    #[error(
        "Cannot kill process {0}.\n\nTroubleshoot:\n  - Check that the process belongs to you\n  - Kill it manually and try again\n"
    )]
    KillFailed(u64),

    #[error("Could not detect shell")]
    CouldNotDetectShell,

    #[error(
        "Environment variable '{0}' is not set.\n\nTemporary (current session):\n  - Linux/macOS: export {0}=value\n  - Windows (PowerShell): $env:{0} = \"value\"\n  - Windows (CMD): set {0}=value\n\nPermanent: Add to your shell profile or Windows Environment Variables settings."
    )]
    UnsetEnvVar(String),

    // File system
    #[error("Cannot {operation} '{}' ({source})", path.display())]
    FileSystem {
        operation: FsOperation,
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Cannot rename file from '{}' to '{}' ({source})", from.display(), to.display())]
    RenameFile {
        from: PathBuf,
        to: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Cannot write directory '{}'", .0.display())]
    ReadOnlyDir(PathBuf),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    // Fallback. Anything that knows a path or a port reports that instead.
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),

    // Network and SSH
    #[error("Cannot connect to port {0} ({1})")]
    ConnectionFailed(u16, #[source] io::Error),

    #[error(
        "No available port found.\n\nAll ports are currently in use. Stop unused processes and try again."
    )]
    NoPortAvailable,

    #[error("Connection to instance '{0}' failed")]
    SshConnectionFailed(String),

    #[error("Authentication on instance '{0}' failed")]
    SshAuthFailed(String),

    #[error("Authentication on instance '{0}' was cancelled")]
    SshAuthCancelled(String),

    #[error("The new SSH host key of instance '{0}' was not trusted")]
    SshHostKeyRejected(String),

    #[error("SSH Error: {0}")]
    Ssh(#[from] russh::keys::ssh_key::Error),

    #[error("Cannot {operation} '{path}' on the instance ({source})")]
    Sftp {
        operation: FsOperation,
        path: String,
        #[source]
        source: russh_sftp::client::error::Error,
    },

    #[error("Cannot open an SFTP session on instance '{instance}' ({source})")]
    SftpSession {
        instance: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Web Error: {0}")]
    Web(#[from] reqwest::Error),

    // TLS
    #[error("TLS certificate generation error: {0}")]
    TlsCertGeneration(#[from] rcgen::Error),

    #[error("TLS connection error: {0}")]
    TlsConnection(#[source] Box<dyn std::error::Error + Send + Sync>),

    // Parsing
    #[error(
        "CPU arch '{0}' is not supported.\n\nChoose a supported architecture: 'amd64' or 'arm64'"
    )]
    UnknownArch(String),

    #[error(
        "Invalid username '{0}'.\n\nUsernames must start with a lowercase letter or underscore, followed by lowercase letters, numbers, underlines or dashes"
    )]
    InvalidUsername(String),

    // Serialization
    #[error("JSON Error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("TOML Error: {0}")]
    SerdeToml(#[from] toml::ser::Error),
}

impl Error {
    pub fn from_fs(operation: FsOperation, path: &Path, source: io::Error) -> Self {
        Error::FileSystem {
            operation,
            path: path.to_path_buf(),
            source,
        }
    }

    pub fn from_sftp(
        operation: FsOperation,
        path: &str,
        source: russh_sftp::client::error::Error,
    ) -> Self {
        Error::Sftp {
            operation,
            path: path.to_string(),
            source,
        }
    }

    pub fn from_config(name: &str, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Error::InvalidInstanceConfig {
            name: name.to_string(),
            source: Box::new(source),
        }
    }

    pub fn from_tls(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Error::TlsConnection(Box::new(source))
    }

    pub fn from_sftp_session(
        instance: &str,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Error::SftpSession {
            instance: instance.to_string(),
            source: Box::new(source),
        }
    }
}

fn format_qemu_not_found_help() -> String {
    let install = if cfg!(target_os = "macos") {
        "  - brew install qemu"
    } else if cfg!(target_os = "windows") {
        "  - winget install SoftwareFreedomConservancy.QEMU
  - or download QEMU from https://www.qemu.org/download/#windows"
    } else {
        "  - Debian/Ubuntu:  sudo apt install qemu-system qemu-utils ovmf qemu-efi-aarch64
  - Fedora/RHEL:    sudo dnf install qemu-system-x86 qemu-img edk2-ovmf edk2-aarch64
  - Arch Linux:     sudo pacman -S qemu-full edk2-ovmf edk2-armvirt
  - openSUSE:       sudo zypper install qemu qemu-tools qemu-ovmf-x86_64 qemu-uefi-aarch64"
    };
    format!(
        "QEMU or its UEFI firmware was not found.

Cubic needs the following to run virtual machines:
  - qemu-system-x86_64  (amd64 VMs)
  - qemu-system-aarch64 (arm64 VMs)
  - qemu-img            (disk image management)
  - UEFI firmware

Install QEMU and its UEFI firmware:
{install}

Or set CUBIC_QEMU_DIR to the directory that contains your QEMU install
(CUBIC_QEMU_FW_AMD64 / CUBIC_QEMU_FW_ARM64 override just the UEFI firmware path).
"
    )
}
