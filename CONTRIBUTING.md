# Contributing to WhatNull

We welcome contributions to WhatNull! To ensure the project remains secure, fast, and high quality, please follow these guidelines.

## Development Rules
- **English Only**: Source code, logs, comments (if any), commits, and documentation must be in English.
- **No Comments**: Write self-documenting code. Avoid adding comment lines (`//` or `/* */`). Use descriptive naming, small functions, and strong typing instead.
- **Tauri Independence**: Keep Rust business logic in independent workspace crates (`crates/`) and out of `apps/desktop/src-tauri` whenever possible.

## Pull Request Process
1. Fork the repository and create a branch.
2. Run quality checks locally:
   - Rust: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`
   - Frontend: `npm run typecheck`, `npm run lint`, `npm run build`
3. Open a pull request describing the changes and verification steps.
