[![Cubic](https://github.com/cubic-vm/cubic/blob/main/cubic.svg)](https://github.com/cubic-vm/cubic)
⭐ Please star us on [GitHub](https://github.com/cubic-vm/cubic) to promote the project!

[![github.com](https://github.com/cubic-vm/cubic/actions/workflows/build.yml/badge.svg)](https://github.com/cubic-vm/cubic/actions/workflows/build.yml)
[![crates.io](https://img.shields.io/crates/v/cubic.svg)](https://crates.io/crates/cubic)
[![MSRV](https://img.shields.io/crates/msrv/cubic.svg)](https://crates.io/crates/cubic)
[![snapcraft.io](https://snapcraft.io/cubic/badge.svg)](https://snapcraft.io/cubic)

Cubic spins up Linux virtual machines on Linux, macOS and Windows with a single
command.

Every distribution comes as a prebuilt cloud image and is ready to use within
seconds, so you skip the long installation. Cubic keeps things simple and secure
by acting as lightweight glue over proven tools. No privileged system service is
required and every VM runs as your normal user, so you never need admin or root
rights.
Cubic is built on top of `QEMU`, `EDK2`, official cloud images and `cloud-init`.

![Cubic Demo](docs/cubic.gif)

# 🧐 Why use Cubic?

One command takes you from nothing to a shell inside a fresh Linux VM. The images
are official and verified, downloaded straight from each distribution. Every
machine is a real VM, so you get stronger isolation than containers can offer.
The same workflow runs on Linux, macOS and Windows across amd64 and arm64.
No privileged system service is required and you never need admin or root rights.

Cubic fits a lot of everyday workflows:

- Spin up disposable VMs that are ready in seconds and easy to throw away
- Compare many Linux distributions side by side on any host
- Develop and test across distributions and CPU architectures
- Try or build software without polluting your host
- Run untrusted or experimental software inside an isolated VM
- Reproduce CI or production Linux environments locally
- Run local services such as HTTP servers and databases with port forwarding

# 🔥 Features

- Simple command-line interface
- Supports the following guest OS:
  - **Alma Linux**
  - **Arch Linux**
  - **Debian**
  - **Fedora**
  - **Gentoo**
  - **OpenSUSE**
  - **Rocky Linux**
  - **Ubuntu**
- Supports the following host OS: **Linux**, **macOS**, **Windows**
- Supports **amd64** and **arm64** CPU architectures
- Supports hardware acceleration with **KVM** (Linux), **Hypervisor** (macOS), and **Hyper-V** (Windows)
- Daemonless design which does not require root privileges
- Written in Rust

# 🚀 Quick Start

A virtual machine instance can be created with a single command. This example
creates an instance from a Ubuntu image with the name `quickstart`.
```
$ cubic run quickstart --image ubuntu:noble
Welcome to Ubuntu 24.04.4 LTS (GNU/Linux 6.8.0-101-generic x86_64)

 * Documentation:  https://help.ubuntu.com
 * Management:     https://landscape.canonical.com
 * Support:        https://ubuntu.com/pro

This system has been minimized by removing packages and content that are
not required on a system that users do not log into.

To restore this content, you can run the 'unminimize' command.

The programs included with the Ubuntu system are free software;
the exact distribution terms for each program are described in the
individual files in /usr/share/doc/*/copyright.

Ubuntu comes with ABSOLUTELY NO WARRANTY, to the extent permitted by
applicable law.

cubic@quickstart:~$
```

Use `cubic images` to list all supported images.

# 💫 How to install Cubic?

**Ubuntu** (Snap)
```
sudo snap install cubic && \
sudo snap connect cubic:kvm
```

**macOS** (Homebrew)
```
brew install cubic-vm/cubic/cubic
```

**Others** (Cargo)

Install the [Rust toolchain](https://rustup.rs) and then build Cubic:
```
cargo install cubic
```

Cubic needs QEMU and its UEFI firmware on the host. Install them with your
package manager:
```
# Debian/Ubuntu
sudo apt install qemu-system qemu-utils ovmf qemu-efi-aarch64
# Fedora/RHEL
sudo dnf install qemu-system-x86 qemu-img edk2-ovmf edk2-aarch64
# Arch Linux
sudo pacman -S qemu-full edk2-ovmf edk2-armvirt
# openSUSE
sudo zypper install qemu qemu-tools qemu-ovmf-x86_64 qemu-uefi-aarch64
# macOS
brew install qemu
# Windows
winget install SoftwareFreedomConservancy.QEMU
```

See the [install](https://cubic-vm.org/install.html) instructions for more information.

# 💡 How to use Cubic?

Cubic has a simple CLI:
```
$ cubic --help
Cubic runs Linux virtual machines on Linux, macOS and Windows with a single
command.

Every distribution comes as a prebuilt cloud image and is ready to use within
seconds, so you skip the long installation. Cubic keeps things simple and secure
by acting as lightweight glue over proven tools. No privileged system service is
required and every VM runs as your normal user, so you never need admin or root
rights. Cubic is built on top of QEMU, EDK2, official cloud images and
cloud-init.

Examples:

  Create a new VM instance with:
  $ cubic create example --image ubuntu:noble
  Open a shell in the VM instance:
  $ cubic ssh example

  Alternatively, use `run` to execute the above commands in a single command:
  $ cubic run example --image ubuntu:noble

  Show all supported VM images:
  $ cubic images

  List previously created VM instances:
  $ cubic instances

  Show information about a VM instance:
  $ cubic show <instance>

  Execute a command in a VM instance:
  $ cubic exec <instance> <shell command>

  Transfer files and directories between host and VM instance:
  $ cubic scp <path/to/host/file> <instance>:<path/to/guest/file>
  See `cubic scp --help` for more examples

For more information, visit: https://cubic-vm.org/
The source code is located at: https://github.com/cubic-vm/cubic

Usage: cubic [OPTIONS] [COMMAND]

Commands:
  run          Create and start VM instances
  create       Create VM instances
  instances    List VM instances
  images       List VM images
  ports        List ports for VM instances
  show         Show VM images and instances
  modify       Modify VM instances
  console      Open VM instance console
  ssh          Connect to VM instances
  scp          Copy data between host and VM instances
  exec         Execute commands on VM instances
  start        Start VM instances
  stop         Stop VM instances
  restart      Restart VM instances
  rename       Rename VM instances
  clone        Clone VM instances
  delete       Delete VM instances
  prune        Clear caches
  completions  Generate shell completion scripts

Options:
  -v, --verbose  Increase logging output
  -q, --quiet    Reduce logging output
  -h, --help     Print help
  -V, --version  Print version
```

# 🔨 How to Build Cubic from Source?

See [CONTRIBUTING.md](CONTRIBUTING.md) for instructions on setting up a development
environment and building the project.

# 💬 How to contribute to Cubic?

We are actively looking for help to improve Cubic. You can help in various ways:

- 👧 Increase Cubic's user base by installing and using it!
- ⭐ Star us on [Github](https://github.com/cubic-vm/cubic) to promote the project!
- 🪲 If you found a bug or you are interested in a feature, please create an [issue on Github](https://github.com/cubic-vm/cubic/issues)!
- 👷 If you are a developer and you want to submit a change, please have a look at the [contribution page](CONTRIBUTING.md)!
- 📝 If you are a technical writer and you want to improve the documentation, please have a look at the [contribution page](CONTRIBUTING.md)!

# 📦 Dependencies

Cubic keeps its dependency tree small and lean, acting as thin glue over proven
tools rather than reinventing them.

At runtime Cubic has a single dependency: **QEMU**, the virtualization engine it
drives to run every VM.

| Rust Crate | Usage |
|------------|-------|
| clap | Parse CLI commands, arguments, and flags |
| clap_complete | Generate shell completion scripts |
| crossterm | Terminal control for the interactive console and views |
| getrandom | Secure randomness for SSH key generation |
| regex | Parse image and instance names and scrape image version listings |
| rcgen | Generate the per-instance self-signed certificates for QEMU mTLS |
| reqwest | Download cloud images over HTTPS |
| russh | Pure-Rust SSH client to connect into VMs |
| russh-sftp | SFTP file transfer over the SSH connection |
| rustls | TLS for the QEMU mTLS control channel |
| rustls-pki-types | Shared certificate and key types backing rustls |
| serde | Derive serialization for config and QMP messages |
| serde_json | QMP protocol and firmware descriptor parsing |
| serde_yaml | Parse cloud-init and instance YAML |
| sha2 | Verify downloaded image checksums |
| sysinfo | Read the host username and detect running QEMU processes |
| thiserror | Derive the crate's error types |
| tokio | Async runtime for SSH and SFTP transfers |
| toml | Read and write the `instance.toml` config |

# 📃 License

Cubic is dual-licensed under [Apache](LICENSE-APACHE) and [MIT](LICENSE-MIT).
