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

The recommended workflow uses **Docker** and **Make**, which build and run all checks inside a pinned container matching the CI environment:

- **Docker**
- **Make**
- **Git**

Alternatively, if you prefer building directly on your host without Docker:

- **Git**
- **GCC**
- **Rustup** (Rust toolchain `1.92.0` matching CI and the container)

For **Debian**, **Ubuntu**, and derivatives:
```bash
sudo apt update && sudo apt install -y git gcc rustup make docker.io
```

For **Fedora** and derivatives:
```bash
sudo dnf install -y git gcc rustup make docker && sudo rustup-init -y
```

For **OpenSUSE** and derivatives:
```bash
sudo zypper install -y git gcc rustup make docker
```

Then clone the repository:
```bash
git clone https://github.com/cubic-vm/cubic.git
cd cubic/
rustup toolchain install 1.92.0
rustup override set 1.92.0
```

## How to build?

The primary and recommended way to build Cubic is using Make, which runs inside the pinned container:

```bash
make build
```

Alternatively, you can build locally with Cargo:

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

The recommended way to run the test suite is via the containerized Make target:

```bash
make test
```

Alternatively, you can run tests directly with Cargo:

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

Using Make, you can build and preview the documentation server inside the container:

```bash
make doc
```

Alternatively, to build on your host:

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

The recommended way to fix formatting across Rust code and configuration files is:

```bash
make fix-format
```

To check formatting without modifying files:
```bash
make format
```

Alternatively, you can format Rust code directly with Cargo:

```bash
cargo fmt
```

To only check without modifying files:
```bash
cargo fmt --check
```

## How to lint?

The recommended way to run lints inside the pinned container is:

```bash
make lint
```

To automatically apply safe fixes:
```bash
make fix-lint
```

You can also run both format and lint fixes together:
```bash
make fix
```

Alternatively, you can run Clippy directly with Cargo:

```bash
cargo clippy --all-targets -- -D warnings
```

To automatically apply safe fixes:
```bash
cargo clippy --all-targets --fix --allow-dirty
```

## How to run a security audit?

The recommended way to run a security audit inside the container is:

```bash
make audit
```

Alternatively, using Cargo directly:

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
  - `perf: ...` for performance improvements
  - `refactor: ...` for code refactorings
  - `style: ...` for formatting or whitespace changes with no logic change
  - `docs: ...` for documentation changes
  - `test: ...` for adding or correcting tests
  - `build: ...` for build system or dependency changes
  - `ci: ...` for CI/CD pipeline changes
  - `chore: ...` for changes not related to source code
  - `revert: ...` for reverting a previous commit

Before opening a pull request, please verify that your changes pass all checks.
The recommended way is running the containerized check suite:

```bash
make check
```

This runs formatting, linting (including yamllint and shellcheck), unit tests, and security audits matching CI.

Alternatively, if running directly with Cargo:
```bash
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && cargo audit
```

## What license does Cubic use?

Cubic is dual-licensed under the MIT and Apache 2.0 licenses.
By submitting a pull request, you agree that your contribution is licensed under these licenses.
