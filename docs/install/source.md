# Install Cubic from Source

Cubic requires the following dependencies:
  - Cargo
  - QEMU
  - OpenSSH Client
  - mkisofs

The dependencies can be installed for Debian and Ubuntu with the following command:
```
$ sudo apt install cargo qemu-system-x86 qemu-system-arm qemu-utils genisoimage openssh-client
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
