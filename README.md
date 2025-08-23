[![Cubic](https://github.com/cubic-vm/cubic/blob/main/cubic.svg)](https://github.com/cubic-vm/cubic)
:star: Please star us on [Github](https://github.com/cubic-vm/cubic) to promote the project!

[![github.com](https://github.com/cubic-vm/cubic/actions/workflows/cubic.yml/badge.svg)](https://github.com/cubic-vm/cubic/actions/workflows/cubic.yml)
![crates.io](https://img.shields.io/crates/v/cubic.svg)
[![snapcraft.io](https://snapcraft.io/cubic/badge.svg)](https://snapcraft.io/cubic)


Cubic is a lightweight command-line manager for virtual machines with focus on simplicity and security.

It has a simple, daemon-less and rootless design. All Cubic virtual machines run isolated in the user context.
Cubic is built on top of `QEMU`, `KVM` and `cloud-init`.

[![Get it from the Snap Store](https://snapcraft.io/en/dark/install.svg)](https://snapcraft.io/cubic)

# :fire: Features

- Simple command-line interface
- Supports ArchLinux, Debian, Fedora, OpenSUSE and Ubuntu guest images
- Supports Linux, macOS and Window hosts with amd64 and arm64 architecture
- Supports hardware acceleration with KVM (Linux), Hypervisoer (macOS) and Hyper-V (Windows)
- Daemon-less design which does not require root rights
- Written in Rust

# :rocket: Quick Start

A virtual machine instance can be created with a single command:
```
$ cubic run quickstart --image ubuntu:noble
Welcome to Ubuntu 24.04 LTS (GNU/Linux 6.8.0-35-generic x86_64)

 * Documentation:  https://help.ubuntu.com
 * Management:     https://landscape.canonical.com
 * Support:        https://ubuntu.com/pro

 System information as of Sun Jul 14 13:58:15 UTC 2024

  System load:            0.15
  Usage of /:             60.7% of 2.35GB
  Memory usage:           29%
  Swap usage:             0%
  Processes:              150
  Users logged in:        0
  IPv4 address for ens13: 10.0.2.15
  IPv6 address for ens13: fec0::5054:ff:fe12:3456

Expanded Security Maintenance for Applications is not enabled.

0 updates can be applied immediately.

Enable ESM Apps to receive additional future security updates.
See https://ubuntu.com/esm or run: sudo pro status


The list of available updates is more than a week old.
To check for new updates run: sudo apt update


The programs included with the Ubuntu system are free software;
the exact distribution terms for each program are described in the
individual files in /usr/share/doc/*/copyright.

Ubuntu comes with ABSOLUTELY NO WARRANTY, to the extent permitted by
applicable law.

cubic@quickstart:~$
```

# :dizzy: How to install Cubic?

### Linux

Install Cubic on Linux as [Snap](https://snapcraft.io) with the following command:
```bash
sudo sh -c "snap install cubic && snap connect cubic:kvm && snap connect cubic:ssh-keys"
```

### macOS

Install Cubic on macOS via [Homebrew](https://brew.sh) with the following command:
```bash
brew install cubic-vm/cubic/cubic
```

### Windows

Install Cubic on Windows in [WSL](https://learn.microsoft.com/en-us/windows/wsl/install) as [Snap](https://snapcraft.io) with the following command:
```bash
sudo sh -c "snap install cubic && snap connect cubic:kvm && snap connect cubic:ssh-keys"
```

### Others

> [!NOTE]  
> Cubic requires the following dependencies: `QEMU`, `OpenSSH` and either `cdrtools` or `cdrkit`.

Install Cubic from [crates.io](https://crates.io/crates/cubic):
```bash
cargo install cubic
```

# :bulb: How to use Cubic?

Cubic has a simple CLI:
```
$ cubic --help
Cubic is a lightweight command line manager for virtual machines. It has a
simple, daemon-less and rootless design. All Cubic virtual machines run
isolated in the user context. Cubic is built on top of QEMU, KVM and cloud-init.

Show all supported images:
$ cubic image ls

Create a new virtual machine instance:
$ cubic create mymachine --image ubuntu:noble

List all virtual machine instances:
$ cubic instances

Start an instance:
$ cubic start <instance name>

Stop an instance:
$ cubic stop <instance name>

Open a shell in the instance:
$ cubic ssh <machine name>

Copy a file from the host to the instance:
$ cubic scp <path/to/host/file> <machine>:<path/to/guest/file>

Copy a file from the instance to the hots:
$ cubic scp <machine>:<path/to/guest/file> <path/to/host/file>


Usage: cubic [COMMAND]

Commands:
  run        Create, start and open a shell in a new virtual machine instance
  create     Create a new virtual machine instance
  instances  List all virtual machine instances
  rm         Delete virtual machine instances
  info       Get information about an virtual machine instance
  console    Open the console of an virtual machine instance
  ssh        Connect to a virtual machine instance with SSH
  scp        Copy a file from or to a virtual machine instance with SCP
  start      Start virtual machine instances
  stop       Stop virtual machine instances
  restart    Restart virtual machine instances
  config     Modify virtual machine instance configuration
  rename     Rename a virtual machine instance
  clone    Clone a virtual machine instance
  image    Image subcommands
  net      Network subcommands
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

# :speech_balloon: How to contribute to Cubic?

We are actievely looking for help to improve Cubic.
You can help in various ways:
- :girl: Increase Cubic's user base by installing and using it!
- :star: Star us on [Github](https://github.com/cubic-vm/cubic) to promote the project!
- :beetle: If you found a bug or you are interested in a feature please create an [issue on Github](https://github.com/cubic-vm/cubic/issues)!
- :construction_worker: If you are a developer and you want to submit a change please have a look at the [contribution page](CONTRIBUTING.md)!

# 📃 License

Cubic is dual-licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT).
