# Cubic

Cubic is a lightweight command line manager for virtual machines with focus on simplicity and security.

It has a simple, daemon-less and rootless design. All Cubic virtual machines run isolated in the user context.
Cubic is built on top of `QEMU`, `KVM` and `cloud-init`.

**Official website**: https://github.com/cubic-vm/cubic

## Features

- Simple command line interface
- Daemon-less design
- Works without root rights
- Supports KVM acceleration
- Supports ArchLinux, Debian, Fedora and Ubuntu guest images
- Supports file transfers between host and guest
- Supports directory mounting between host and guest
- Written in Rust

## Quick Start

A virtual machine instance can be created with a single command:
```
$ cubic run --name quickstart --image ubuntu:noble
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

The supported images can be listed with:
```
$ cubic image list --all
Vendor      Version  Name         Arch       Size
archlinux    latest  latest      amd64
debian            9  stretch     amd64
debian           10  buster      amd64
debian           11  bullseye    amd64
debian           12  bookworm    amd64
debian           13  trixie      amd64
debian           14  forky       amd64
fedora           39  39          amd64
fedora           40  40          amd64
fedora           41  41          amd64
ubuntu        18.04  bionic      amd64
ubuntu        18.10  cosmic      amd64
ubuntu        19.04  disco       amd64
ubuntu        19.10  eoan        amd64
ubuntu        20.04  focal       amd64
ubuntu        20.10  groovy      amd64
ubuntu        21.04  hirsute     amd64
ubuntu        21.10  impish      amd64
ubuntu        22.04  jammy       amd64  284.6 MiB
ubuntu        22.10  kinetic     amd64
ubuntu        23.04  lunar       amd64
ubuntu        23.10  mantic      amd64
ubuntu        24.04  noble       amd64
ubuntu        24.10  oracular    amd64
```

A virtual machine instance can be started, stopped and restarted by:
- `cubic start <instance name>`
- `cubic stop <instance name>`
- `cubic restart <instance name>`

In order to open a shell in the virtual machine instance use:
```cubic ssh <machine name>```

## How to install Cubic?
- [Install Cubic as Snap](docs/install/snap.md)
- [Install Cubic from source](docs/install/source.md)

## How to use Cubic?

Cubic has a simple CLI:
```
$ cubic --help
Cubic is a lightweight command line manager for virtual machines. It has a
simple, daemon-less and rootless design. All Cubic virtual machines run
isolated in the user context. Cubic is built on top of QEMU, KVM and cloud-init.

Show all supported images:
$ cubic image list --all

Create a new virtual machine instance:
$ cubic instance add --name mymachine --image ubuntu:noble

List all virtual machine instances:
$ cubic list

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
  run       Setup and run a new instance
  list      List instances
  info      Get information about an instance
  sh        Open a shell in an instance
  ssh       Connect to an instance with SSH
  scp       Copy a file from or to an instance with SCP
  start     Start instances
  stop      Stop instances
  restart   Restart instances
  instance  Instance commands
  image     Image commands
  mount     Mount commands
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Usage:
- [Add and Delete Virtual Machines](docs/usage/add_delete.md)
- [Start, Stop and Restart Virtual Machines](docs/usage/start_stop.md)
- [List Images and Virtual Machines](docs/usage/list.md)
- [Open Shell in Virtual Machines](docs/usage/sh.md)
- [Transfer Directories and Files](docs/usage/copy_mount.md)
- [Configure Virtual Machines](docs/usage/configure.md)
- [Rename and Clone Virtual Machines](docs/usage/rename_clone.md)

## How to contribute to Cubic?

See: [Contribute to Cubic](CONTRIBUTING.md)
