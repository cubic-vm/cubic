[![Cubic](https://github.com/cubic-vm/cubic/blob/main/cubic.svg)](https://github.com/cubic-vm/cubic)
:star: Please star us on [GitHub](https://github.com/cubic-vm/cubic) to promote the project!

[![github.com](https://github.com/cubic-vm/cubic/actions/workflows/build.yml/badge.svg)](https://github.com/cubic-vm/cubic/actions/workflows/build.yml)
[![crates.io](https://img.shields.io/crates/v/cubic.svg)](https://crates.io/crates/cubic)
[![snapcraft.io](https://snapcraft.io/cubic/badge.svg)](https://snapcraft.io/cubic)


Cubic is a lightweight command-line manager for virtual machines with a focus on simplicity and security.

It has a simple, daemonless, and rootless design. All Cubic virtual machines run isolated in the user context.
Cubic is built on top of `QEMU`, `KVM`, and `cloud-init`.

[![Get it from the Snap Store](https://snapcraft.io/en/dark/install.svg)](https://snapcraft.io/cubic)

# :monocle_face: Why use Cubic?

Cubic is a simple tool that may be used for various purposes, such as:

- Developing and testing software on different Linux distributions
- Installing software on a virtual machine to avoid polluting the host system
- Running untrusted software on an isolated virtual machine

# :fire: Features

- Simple command-line interface
- Supports the following guest OS:
  - **ArchLinux**
  - **Debian**
  - **Fedora**
  - **OpenSUSE**
  - **RockyLinux**
  - **Ubuntu**
- Supports the follwoing host OS: **Linux**, **macOS**, **Windows**
- Supports **amd64** and **arm64** CPU architectures
- Supports hardware acceleration with **KVM** (Linux), **Hypervisor** (macOS), and **Hyper-V** (Windows)
- Daemonless design which does not require root privileges
- Written in Rust

# :rocket: Quick Start

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

# :dizzy: How to install Cubic?

See the [install](https://cubic-vm.org/install.html) instructions for more information.

# :bulb: How to use Cubic?

Cubic has a simple CLI:
```
$ cubic --help
Cubic is a lightweight command line manager for virtual machines. It has a
simple, daemonless and rootless design. All Cubic virtual machines run isolated
in the user context. Cubic is built on top of QEMU, KVM and cloud-init.

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


Usage: cubic [OPTIONS] <COMMAND>

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

# :hammer: How to Build Cubic from Source?

## Install Toolchain

Before running the build commands, ensure you have the necessary tools installed:

- **Git**
- **GCC**
- **Rustup**

For **Debian**, **Ubuntu**, and derivatives:
```bash
sudo apt update && sudo apt install -y git gcc rustup
```

For **Fedora** and derivatives:
```bash
sudo dnf install -y git gcc rustup && sudo rustup-init -y
```

For **OpenSUSE** and derivatives:
```bash
sudo zypper install -y git gcc rustup
```

## Build Cubic

Download the source code, navigate to the Cubic source directory and run the build command.

```bash
git clone https://github.com/cubic-vm/cubic.git
cd cubic/
rustup toolchain add stable
cargo build --locked --release
```
The target executable is located at `target/release/cubic`.

**Note**:
- The `--release` flag is used to create an optimized version of the application.
- The `--locked` flag is used to ensure the build uses the exact dependency versions intended by the developers.

## Runtime Dependencies

Once built, Cubic needs these tools to actually run the virtual machines:

- **QEMU** (qemu-system-x86_64, qemu-system-arm64, qemu-img)
- **cdrtools** or **cdrkit** (mkisofs)


# :speech_balloon: How to contribute to Cubic?

We are actively looking for help to improve Cubic. You can help in various ways:

- :girl: Increase Cubic's user base by installing and using it!
- :star: Star us on [Github](https://github.com/cubic-vm/cubic) to promote the project!
- :beetle: If you found a bug or you are interested in a feature, please create an [issue on Github](https://github.com/cubic-vm/cubic/issues)!
- :construction_worker: If you are a developer and you want to submit a change, please have a look at the [contribution page](CONTRIBUTING.md)!

# :page_with_curl: License

Cubic is dual-licensed under [Apache](LICENSE-APACHE) and [MIT](LICENSE-MIT).
