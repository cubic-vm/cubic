# Cubic

Cubic is a lightweight command line manager for virtual machines.

## Quick Start

Create and run a virtual machine:
```
$ cubic run --name quickstart --image ubuntu:jammy
Welcome to Ubuntu 22.04.4 LTS (GNU/Linux 5.15.0-107-generic x86_64)

 * Documentation:  https://help.ubuntu.com
 * Management:     https://landscape.canonical.com
 * Support:        https://ubuntu.com/pro

 System information as of Sun Jun  2 19:30:36 UTC 2024

  System load:           0.75
  Usage of /:            72.3% of 1.96GB
  Memory usage:          22%
  Swap usage:            0%
  Processes:             93
  Users logged in:       0
  IPv4 address for ens3: 10.0.2.15
  IPv6 address for ens3: fec0::5054:ff:fe12:3456

Expanded Security Maintenance for Applications is not enabled.

0 updates can be applied immediately.

Enable ESM Apps to receive additional future security updates.
See https://ubuntu.com/esm or run: sudo pro status



The programs included with the Ubuntu system are free software;
the exact distribution terms for each program are described in the
individual files in /usr/share/doc/*/copyright.

Ubuntu comes with ABSOLUTELY NO WARRANTY, to the extent permitted by
applicable law.

cubic@quickstart:~$
```

## Install Cubic

Cubic can be installed from the Snap Store:
```
$ sudo snap install cubic
```
Permit access to the kernel virtual machine (KVM) for hardware acceleration:
```
$ sudo snap connect cubic:kvm
```

## Cubic Usage

Cubic has a simple interface:
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
  attach   Attach to serial console
  ssh      Connect to a machine with SSH
  scp      Copy a file from or to a machine with SCP
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Create a virtual machine:
```
$ cubic add --name example --image ubuntu:jammy:amd64 --cpus 4 --mem 4G --disk 5G
```

List all virtual machines:
```
$ cubic list
Name             CPUs     Memory       Disk  State     
quickstart          1    1.0 GiB    1.0 GiB  RUNNING 
example             4    4.0 GiB    5.0 GiB  STOPPED 
```

Start a virtual machine:
```
$ cubic start example
```

Connect with SSH to a virtual machine:
```
$ cubic ssh example
```

Copy a file with SSH to a virtual machine:
```
$ touch test
$ cubic scp test example:~/
```

Restart a virtual machine:
```
$ cubic restart example 
```

Stop a virtual machine:
```
$ cubic stop example 
```

Show a virtual machine config:
```
$ cubic config example 
cpus: 4 
mem:  4.0 GiB
disk: 2.2 GiB
```
Change a virtual machine config:
```
$ cubic config --cpus 5 --mem 5G --disk 5G example
cpus: 5 
mem:  5.0 GiB
disk: 5.0 GiB
```

Clone a virtual machine:
```
$ cubic clone example example2
```

Rename a virtual machine:
```
$ cubic rename example2 example_new
```

Delete a virtual machine:
```
$ cubic delete example_new
```

List all images:
```
$ cubic list images
ID                    ARCH      SIZE
ubuntu:jammy         amd64   2.2 GiB
```

## Build Cubic from Source

Cubic requires the following dependencies:
  - Cargo
  - QEMU
  - Bubblewrap
  - Cloud Utils
  - Wget
  - OpenSSH Client

The dependencies can be installed for Debian and Ubuntu with the following command:
```
$ sudo apt install cargo wget qemu-system-x86 bubblewrap cloud-image-utils openssh-client
```

Build the Rust project with the Cargo package manager:
```
$ cargo build
```

Install the binaries:
```
$ cargo install --path .
$ export PATH="$PATH:$HOME/.cargo/bin"
```

## Contribute to Cubic

Please make sure that any contributed code is correctly format, linted and tested.

Format source code:
```
$ cargo fmt
```

Lint code:
```
$ cargo clippy
```

Run tests:
```
$ cargo test
```
