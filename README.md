# <img src="docs/logo-mark.svg" alt="Cubic logo" height="64" align="absmiddle"> Cubic | Linux VMs in one command

[![github.com](https://github.com/cubic-vm/cubic/actions/workflows/build.yml/badge.svg)](https://github.com/cubic-vm/cubic/actions/workflows/build.yml)
[![crates.io](https://img.shields.io/crates/v/cubic.svg)](https://crates.io/crates/cubic)
[![MSRV](https://img.shields.io/crates/msrv/cubic.svg)](https://crates.io/crates/cubic)
[![snapcraft.io](https://snapcraft.io/cubic/badge.svg)](https://snapcraft.io/cubic)

![Cubic demo that creates and enters a VM instance](docs/cubic.gif)

Cubic is a cross-platform tool that spins up Linux virtual machines with a
single command. It is made for developers who need a clean Linux system fast.

Linux distributions come as official images and are ready to use within
seconds. Cubic keeps things simple and secure by acting as lightweight glue over
proven tools such as QEMU. No privileged system service is required and every
VM instance runs as your normal user.

## Install

See the [install guide](https://cubic-vm.org/docs/howto/install.html) for the full
instructions.

**Ubuntu**
```
sudo snap install cubic && sudo snap connect cubic:kvm
```

**macOS**
```
brew install cubic-vm/cubic/cubic
```

**Windows**
```
winget install cubic-vm.cubic
```

**Cargo**
```
cargo install cubic
```

Then create your first VM instance with `cubic run -i ubuntu`.

## Common Commands

These are the most common commands, shown for a VM instance named `demo`.

| Command | What it does |
|---------|--------------|
| `cubic run demo -i ubuntu` | Create the VM instance `demo`, start it and open a shell |
| `cubic run --rm -i ubuntu` | Run a VM instance that is deleted on exit |
| `cubic images` | List the available images |
| `cubic instances` | List your VM instances |
| `cubic ssh demo` | Open a shell in a VM instance |
| `cubic exec demo -- uname -a` | Run one command in a VM instance |
| `cubic scp notes.txt demo:~/` | Copy files between host and VM instance |
| `cubic modify demo -c 4 -m 8G` | Change the vCPUs and memory of a VM instance |
| `cubic snapshot demo/clean` | Save the disk of a VM instance |
| `cubic restore demo/clean` | Roll a VM instance back to a snapshot |
| `cubic clone demo demo2` | Copy a VM instance under a new name |
| `cubic stop demo` | Stop a VM instance |
| `cubic delete demo` | Delete a VM instance |

Run `cubic --help` for the full list or read the
[command reference](https://cubic-vm.org/docs/reference/commands/cubic.html).

## Features

- [Creates a VM instance and opens a shell](https://cubic-vm.org/docs/tutorial/getting_started.html) in one command
- [Runs on Linux, macOS and Windows hosts](https://cubic-vm.org/docs/howto/install.html) with [hardware acceleration](https://cubic-vm.org/docs/explanation/machine.html#hardware-acceleration)
- [Forwards ports](https://cubic-vm.org/docs/howto/ports.html), [copies files](https://cubic-vm.org/docs/howto/copy_files.html) and [executes commands](https://cubic-vm.org/docs/howto/exec.html) in a VM instance
- Supports [templates](https://cubic-vm.org/docs/howto/templates.html), [snapshots](https://cubic-vm.org/docs/howto/snapshots.html) and [clones](https://cubic-vm.org/docs/reference/commands/clone.html) of VM instances
- Runs [temporary](https://cubic-vm.org/docs/howto/temporary_vm.html) and [network isolated](https://cubic-vm.org/docs/tutorial/isolate.html) VM instances
- Runs every VM instance as your normal user [without a privileged system service](https://cubic-vm.org/docs/explanation/security.html#no-privileged-system-service)
- [Verifies every image](https://cubic-vm.org/docs/explanation/security.html#verified-distribution-images) and protects every VM instance with [its own SSH key](https://cubic-vm.org/docs/explanation/security.html#ssh-access)

## Supported Guest Images

- AlmaLinux
- Alpine Linux
- Arch Linux
- CentOS Stream
- Debian
- Fedora
- Gentoo
- openSUSE Leap
- openSUSE Tumbleweed
- Rocky Linux
- Ubuntu

## Documentation

The full documentation is at [cubic-vm.org/docs](https://cubic-vm.org/docs/).
The [Getting Started](https://cubic-vm.org/docs/tutorial/getting_started.html)
tutorial is the best place to begin.

## Contribute

Contributions are very welcome. You can help in many ways:

- Star the project on [GitHub](https://github.com/cubic-vm/cubic)
- Vote with a thumbs up for the [issues](https://github.com/cubic-vm/cubic/issues) you care about
- Report a bug or request a feature in a [new issue](https://github.com/cubic-vm/cubic/issues/new/choose)
- Ask a question or share an idea in the [discussions](https://github.com/cubic-vm/cubic/discussions)
- Fix or improve the documentation, see [CONTRIBUTING.md](CONTRIBUTING.md)
- Send a pull request, see [CONTRIBUTING.md](CONTRIBUTING.md)

## License

Cubic is dual-licensed under [Apache](LICENSE-APACHE) and [MIT](LICENSE-MIT).
