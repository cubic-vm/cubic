# Cubic

Cubic is a lightweight command line manager for virtual machines with focus on simplicity and security.

It has a daemon-less and rootless design. All Cubic virtual machines run unprivileged in the user context.
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

Add, start and open a shell in a new virtual machine:
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

## How to install Cubic?
- [Install Cubic as Snap](docs/install/snap.md)

## How to use Cubic?

Cubic has a simple CLI:
```
$ cubic
Cubic is a lightweight command line manager for virtual machines

Usage: cubic [COMMAND]

Commands:
  run      Add and start a new machine
  add      Add an image or a machine
  delete   Delete images and machines
  clone    Clone a machine
  rename   Rename a machine
  config   Read and write configuration parameters
  list     List images and machines
  start    Start machines
  stop     Stop machines
  restart  Restart a machine
  sh       Open a shell in the machine
  ssh      Connect to a machine with SSH
  scp      Copy a file from or to a machine with SCP
  mount    Mount host directory to guest
  umount   Unmount guest directory
  help     Print this message or the help of the given subcommand(s)

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

## How to build Cubic?

See: [Build Cubic from source](docs/build.md)

## How to contribute to Cubic?

See: [Contribute to Cubic](CONTRIBUTING.md)
