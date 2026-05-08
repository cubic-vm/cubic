# Contribute to Cubic

Contributions are very welcome.
Thank you for taking the time to contribute to Cubic!

## How to contribute?

There are many ways to contribute, no matter your background:

- **Report bugs** — open an [issue on GitHub](https://github.com/cubic-vm/cubic/issues) if something does not work as expected.
- **Request features** — open an [issue on GitHub](https://github.com/cubic-vm/cubic/issues) to suggest improvements.
- **Improve documentation** — fix typos, clarify explanations, or add missing guides in the `docs/` directory.
- **Submit code changes** — open a [pull request](https://github.com/cubic-vm/cubic/pulls) with your fix or feature.
- **Join the discussion** — share feedback and ideas on existing issues and pull requests.

## How to set up a development environment?

Before building Cubic, ensure you have the necessary tools installed:

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

Then clone the repository:
```bash
git clone https://github.com/cubic-vm/cubic.git
cd cubic/
rustup toolchain add 1.92.0
```

## How to build?

Debug build (fast compile, no optimisations):
```bash
cargo build
```

Release build (optimised, matches the distributed binary):
```bash
cargo build --locked --release
```

The release binary is written to `target/release/cubic`.

> **Note:** `--locked` ensures the build uses the exact dependency versions
> intended by the developers.

## How to run the binary?

```bash
cargo run -- [COMMAND] [OPTIONS]
```

For example:
```bash
cargo run -- images
cargo run -- --help
```

### Runtime dependencies

To actually run virtual machines, Cubic requires QEMU to be installed on the host:

- `qemu-system-x86_64`
- `qemu-system-aarch64`
- `qemu-img`

## How to test?

```bash
cargo test
```

To run a single test by name:
```bash
cargo test <test_name>
```

## How to generate the documentation?

The documentation is built with [Sphinx](https://www.sphinx-doc.org/).
The source files live in the `docs/` directory and are written in
[reStructuredText](https://www.sphinx-doc.org/en/master/usage/restructuredtext/basics.html).

First, generate the CLI reference pages from the binary's help output:
```bash
./scripts/generate-docs.sh dev
```

Then build the HTML site:
```bash
sphinx-build docs target/doc
```

The output is written to `target/doc/`.

## How to fix code formatting?

```bash
cargo fmt
```

To only check without modifying files:
```bash
cargo fmt --check
```

## How to lint?

```bash
cargo clippy -- -D warnings
```

To automatically apply safe fixes:
```bash
cargo clippy --fix --allow-dirty
```

## How to run a security audit?

```bash
cargo audit
```

## How to create a good pull request?

High quality pull requests are easier to review and thus take less of your and our time.

General guideline:
- Each pull request must have exactly one intent (fix a bug, update doc, etc.).
- Each pull request should have one Git commit (not mandatory, but recommended).
- Each Git commit must have a descriptive message that explains the changes.
- Each Git commit must have a sign-off (`git commit --signoff`), which
  indicates that you agree with the [Developer Certificate of Origin](https://developercertificate.org/).
- Each Git commit message must start with either:
  - `feat: ...` for features
  - `fix: ...` for bug and security fixes
  - `refactor: ...` for code refactorings
  - `docs: ...` for documentation changes
  - `chore: ...` for changes not related to source code
  - `revert: ...` for reverting a previous commit

Before opening a pull request, please verify that your changes pass all checks:
```bash
cargo fmt --check && cargo clippy -- -D warnings && cargo test && cargo audit
```

## What license does Cubic use?

Cubic is dual-licensed under the MIT and Apache 2.0 licenses.
By submitting a pull request, you agree that your contribution is licensed under these licenses.
