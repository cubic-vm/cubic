# Contribute to Cubic

Contributions are very welcome.
Thank you for taking the time to contribute to Cubic!

## Development Environment

Cubic requires Rust 1.92 or newer. Any installation method works, such
as a package manager or [rustup](https://rustup.rs/). With rustup, the
matching version is selected automatically from `rust-toolchain.toml`.
Some platforms also require a C compiler for linking.

The source code is available on GitHub:

```bash
git clone https://github.com/cubic-vm/cubic.git
cd cubic/
```

For building only, the
[latest release](https://github.com/cubic-vm/cubic/releases/latest)
also provides the source code as an archive.

To build Cubic:

```bash
cargo build --locked --release
```

The binary is written to `target/release/cubic` (`cubic.exe` on Windows).
Running virtual machines also requires QEMU on the host, as described in
the [install guide](https://cubic-vm.org/docs/howto/install.html).

## Checks

Every pull request must pass the following checks.

### Test

```bash
cargo test
```

### Lint

```bash
cargo clippy --all-targets -- -D warnings
```

### Format

```bash
cargo fmt --check
```

### Audit

```bash
cargo audit
```

## Documentation

The documentation is built with [Sphinx](https://www.sphinx-doc.org/).
The source files live in the `docs/` directory and are written in
[reStructuredText](https://www.sphinx-doc.org/en/master/usage/restructuredtext/basics.html).

## Pull Requests

These guidelines help us review your pull request quickly and get it merged sooner.

General guideline:
- Each pull request must have exactly one intent (fix a bug, update doc, etc.).
- Each pull request should have one Git commit (not mandatory, but recommended).
- Each pull request must be read and understood by a human before it is opened.
  Pull requests without the ticked human review box may be closed without review.
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

Before opening a pull request, verify that all [checks](#checks) pass.

By submitting a pull request, the contribution is licensed under the MIT
and Apache 2.0 licenses.
