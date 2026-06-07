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
        "Instance name '{0}' is already taken.\n\nOptions:\n  - Choose a different name\n  - Connect to existing instance: `cubic ssh {0}`"
    )]
    InstanceAlreadyExists(String),

    #[error(
        "Environment variable '{0}' is not set.\n\nTemporary (current session):\n  - Linux/macOS: export {0}=value\n  - Windows (PowerShell): $env:{0} = \"value\"\n  - Windows (CMD): set {0}=value\n\nPermanent: Add to your shell profile or Windows Environment Variables settings."
    )]
    UnsetEnvVar(String),

    #[error(
        "QEMU not found. Cubic requires QEMU to create and run virtual machines.

Missing executable:

{0}

Troubleshoot:
  - Install QEMU for your platform
  - Make sure '{0}' is available on your PATH
  - Linux/WSL2: install the qemu-system and qemu-utils packages for your distribution
  - macOS: install QEMU with Homebrew: `brew install qemu`
  - Or set CUBIC_QEMU=/path/to/binary to override the QEMU binary location

After installing QEMU, rerun your Cubic command.
"
    )]
    QemuNotFound(String),

    #[error(
        "qemu-img not found. Cubic requires qemu-img to manage virtual machine disk images.

Troubleshoot:
  - Install QEMU for your platform (qemu-img is included with QEMU)
  - Make sure 'qemu-img' is available on your PATH
  - Linux/WSL2: install the qemu-utils package for your distribution
  - macOS: install QEMU with Homebrew: `brew install qemu`
  - Or set CUBIC_QEMU_IMG=/path/to/qemu-img to override the qemu-img binary location

After installing qemu-img, rerun your Cubic command.
"
    )]
    QemuImgNotFound,

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
    Ssh(#[from] ssh_key::Error),

    #[error(
        "OVMF firmware not found. Cubic requires UEFI firmware to run AMD64 virtual machines.

Install it with:
  - Debian/Ubuntu:  sudo apt install ovmf
  - Fedora/RHEL:    sudo dnf install edk2-ovmf
  - Arch Linux:     sudo pacman -S edk2-ovmf
  - openSUSE:       sudo zypper install qemu-ovmf-x86_64
  - macOS:          brew install qemu
  - Windows:        install QEMU from https://www.qemu.org/download/#windows
  - Or set CUBIC_FW=/path/to/firmware.fd to override the firmware path

After installing, rerun your Cubic command.
"
    )]
    OvmfNotFound,

    #[error(
        "ARM64 EFI firmware not found. Cubic requires UEFI firmware to run ARM64 virtual machines.

Install it with:
  - Debian/Ubuntu:  sudo apt install qemu-efi-aarch64
  - Fedora/RHEL:    sudo dnf install edk2-aarch64
  - Arch Linux:     sudo pacman -S edk2-armvirt
  - macOS:          brew install qemu
  - Windows:        install QEMU from https://www.qemu.org/download/#windows
  - Or set CUBIC_FW=/path/to/firmware.fd to override the firmware path

After installing, rerun your Cubic command.
"
    )]
    ArmFirmwareNotFound,

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error(
        "No available port found.\n\nAll ports are currently in use. Stop unused processes and try again."
    )]
    NoPortAvailable,

    #[error("SFTP Error: {0}")]
    Sftp(String),
}
