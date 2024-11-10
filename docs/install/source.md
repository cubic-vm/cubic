# Install Cubic from Source

Cubic requires the following dependencies:
  - Cargo
  - QEMU
  - Cloud Utils
  - OpenSSH Client

The dependencies can be installed for Debian and Ubuntu with the following command:
```
$ sudo apt install cargo openssl libssl-dev pkg-config ca-certificates qemu-system-x86 cloud-image-utils openssh-client
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
