use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(
        "CPU arch '{0}' is not supported.\n\nChoose a supported architecture: 'amd64' or 'arm64'"
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
        "Timed out waiting for instance(s) to start.\n\nTroubleshoot:\n  - Run with --verbose to see the QEMU command\n  - Check that QEMU can open /dev/kvm and firmware files\n  - Try again; the system may be under load\n"
    )]
    StartTimeout,

    #[error(
        "Not enough free memory to start instance '{0}'.\n\nTroubleshoot:\n  - Free up memory by stopping other instances or processes\n  - Reduce the instance memory: `cubic modify {0} --memory <size>`\n  - Accept the proposed smaller size by running with --yes\n"
    )]
    NotEnoughMemory(String),

    #[error(
        "Instance name '{0}' is already taken.\n\nOptions:\n  - Choose a different name\n  - Connect to existing instance: `cubic ssh {0}`"
    )]
    InstanceAlreadyExists(String),

    #[error(
        "Environment variable '{0}' is not set.\n\nTemporary (current session):\n  - Linux/macOS: export {0}=value\n  - Windows (PowerShell): $env:{0} = \"value\"\n  - Windows (CMD): set {0}=value\n\nPermanent: Add to your shell profile or Windows Environment Variables settings."
    )]
    UnsetEnvVar(String),

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

    #[error("TLS certificate generation error: {0}")]
    TlsCertGeneration(String),

    #[error("TLS connection error: {0}")]
    TlsConnection(String),

    #[error("Web Error: {0}")]
    Web(#[from] reqwest::Error),

    #[error("JSON Error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("TOML Error: {0}")]
    SerdeToml(#[from] toml::ser::Error),

    #[error("Verification of image failed")]
    InvalidChecksum,

    #[error("Could not detect shell")]
    CouldNotDetectShell,

    #[error("SSH Error: {0}")]
    Ssh(#[from] russh::keys::ssh_key::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error(
        "No available port found.\n\nAll ports are currently in use. Stop unused processes and try again."
    )]
    NoPortAvailable,

    #[error("SFTP Error: {0}")]
    Sftp(String),
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
