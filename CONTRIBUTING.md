# Contributing to Launcher Studio

Thank you for your interest in contributing! This document describes how to set up your environment, the contribution workflow, and the project standards.

Please also read our [Code of Conduct](CODE_OF_CONDUCT.md) and [Security Policy](SECURITY.md) before contributing.

## Getting Started

### Prerequisites
- Rust (latest stable)
- GTK4 development libraries

### Build and Run
```sh
git clone https://github.com/MrArnaudMichel/launcher_studio.git
cd launcher_studio
cargo run
```
For release builds:
```sh
cargo build --release
./target/release/launcher_studio
```

## Development Workflow
1. Fork the repository and create your feature branch from `main` (or the default branch):
   - Branch naming: `feat/short-description`, `fix/short-description`, `docs/short-description`.
2. Make your changes with clear, focused commits.
3. Run and test locally. Ensure the app builds without warnings for your target platform(s).
4. Open a Pull Request (PR):
   - Describe the problem and solution.
   - Include screenshots or recordings for UI changes when feasible.
   - Reference related issues with `Fixes #<id>` when applicable.

## Coding Standards
- Rust edition: per Cargo.toml (2025).
- Use `rustfmt` and `clippy` where possible:
  ```sh
  cargo fmt
  cargo clippy --all-targets --all-features -- -D warnings
  ```
- Write clear, self-documenting code. Add comments where intent is not obvious.
- Keep UI strings user-friendly and, where applicable, consider localization.

## Commit Messages
- Use imperative mood: "Add X", "Fix Y".
- Keep subject <= 72 chars; add a body when more context is needed.
- Reference issues or PRs in the body when relevant.

## Filing Issues
- Use a clear title and description.
- Include steps to reproduce, expected vs actual behavior, and environment details.
- For security issues, do NOT file a public issue—email us as described in [SECURITY.md](SECURITY.md).

## Code Review
- Be responsive to feedback and open to discussion.
- Small, focused PRs are easier to review and merge quickly.

## Documentation
- Update README.md and in-line comments when behavior or usage changes.
- For user-facing changes, consider adding or updating docs and screenshots.

## License
By contributing, you agree that your contributions will be licensed under the project's existing [MIT License](LICENSE).
