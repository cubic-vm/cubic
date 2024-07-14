# Contribute to Cubic

Contribution are very welcome.
Thank you for taking the time to contribute to Cubic!

## How to contribute?

You can contribute by creating a pull request (PR) on the official Github repository:
https://github.com/rogkne/cubic/pulls

## What license is Cubic using?

Cubic is licensed under GPL-2 and any contribution must be released with the same license.

## How to create a good pull request?

High quality pull requests are easier to review and thus take less of your and our time.

General guideline:
- Each pull request must have exactly one intend (fix a bug, update doc, etc.).
- Each pull request should have one Git commit (not mandatory, but recommend).
- Each Git commit must have a descriptive message that explains the changes.
- Each Git commit must have a sign off (git commit --signoff).
- Each Git commit message must start with either:
  - `feat: ...` for features
  - `fix: ...` for bug and security fixes
  - `refactor: ...` for code refactorings
  - `docs: ...` for documentation changes
  - `chore: ...` for changes not related to source code
  - `revert: ...` for reverting a previous commit

Mandatory checks before creating the pull request:
- Build source code: `cargo build`
- Format source code: `cargo fmt`
- Lint source code: `cargo clippy`
- Run tests: `cargo test`
