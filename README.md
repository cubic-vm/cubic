[![Cubic](https://github.com/cubic-vm/cubic/blob/main/cubic.svg)](https://github.com/cubic-vm/cubic)
:star: Please star us on [Github](https://github.com/cubic-vm/cubic) to promote the project!

[![github.com](https://github.com/cubic-vm/cubic/actions/workflows/cubic.yml/badge.svg)](https://github.com/cubic-vm/cubic/actions/workflows/cubic.yml)
![crates.io](https://img.shields.io/crates/v/cubic.svg)
[![snapcraft.io](https://snapcraft.io/cubic/badge.svg)](https://snapcraft.io/cubic)


Cubic is a lightweight command-line manager for virtual machines with focus on simplicity and security.

It has a simple, daemon-less and rootless design. All Cubic virtual machines run isolated in the user context.
Cubic is built on top of `QEMU`, `KVM` and `cloud-init`.

[![Get it from the Snap Store](https://snapcraft.io/en/dark/install.svg)](https://snapcraft.io/cubic)

# :monocle_face: Why use Cubic?

Cubic is a simple tool that may be used for various purposes as for example:

- Develop and test software on different Linux distributions
- Install software on a virtual machine to not populate the host system
- Run untrusted software on an isolated virtual machine

# :fire: Features

- Simple command-line interface
- Supports ArchLinux, Debian, Fedora, OpenSUSE and Ubuntu guest images
- Supports Linux, macOS and Window hosts with amd64 and arm64 architecture
- Supports hardware acceleration with KVM (Linux), Hypervisoer (macOS) and Hyper-V (Windows)
- Daemon-less design which does not require root rights
- Written in Rust

# :rocket: Quick Start

A virtual machine instance can be created with a single command. This example
creates an instance from a Ubuntu image with the name `quickstart`.
```
$ cubic run --image ubuntu:noble quickstart
Downloading ubuntu:24.04:amd64  247.9 MiB [======================================] 100% @  39 Mbps
Successfully verified image checksum
Welcome to Ubuntu 24.04.3 LTS (GNU/Linux 6.8.0-87-generic x86_64)

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

See https://cubic-vm.org/install.html for more information.

# :bulb: How to use Cubic?

Cubic has a simple CLI:
```
$ cubic --help
Cubic is a lightweight command line manager for virtual machines. It has a
simple, daemon-less and rootless design. All Cubic virtual machines run
isolated in the user context. Cubic is built on top of QEMU, KVM and cloud-init.

Show all supported images:
$ cubic images

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


Usage: cubic [OPTIONS] <COMMAND>

Commands:
  run        Create, start and open a shell in a new virtual machine instance
  create     Create a new virtual machine instance
  instances  List all virtual machine instances
  images     List all supported virtual machine images
  ports      List forwarded ports for all virtual machine instances
  show       Show virtual machine image or instance information
  modify     Modify a virtual machine instance configuration
  console    Open the console of an virtual machine instance
  ssh        Connect to a virtual machine instance with SSH
  scp        Copy a file from or to a virtual machine instance with SCP
  start      Start virtual machine instances
  stop       Stop virtual machine instances
  restart    Restart virtual machine instances
  rename     Rename a virtual machine instance
  clone      Clone a virtual machine instance
  delete     Delete one or more virtual machine instances
  prune      Clear cache and free space
  help       Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose  Increase logging output
  -q, --quiet    Reduce logging output
  -h, --help     Print help
  -V, --version  Print version
```

# :speech_balloon: How to contribute to Cubic?

We are actively looking for help to improve Cubic.
You can help in various ways:
- :girl: Increase Cubic's user base by installing and using it!
- :star: Star us on [Github](https://github.com/cubic-vm/cubic) to promote the project!
- :beetle: If you found a bug or you are interested in a feature please create an [issue on Github](https://github.com/cubic-vm/cubic/issues)!
- :construction_worker: If you are a developer and you want to submit a change please have a look at the [contribution page](CONTRIBUTING.md)!

# 📃 License

Cubic is dual-licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT).
