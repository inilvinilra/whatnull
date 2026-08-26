# Testing Strategy

This document details the test suites and validations run inside WhatNull.

## Unit Testing
Rust library tests validate the following:
- Config: atomic JSON writes, path validation, default settings.
- Security: NavigationPolicy evaluation checks.
- Platform: XDG directories path resolution.

To run library tests:
```bash
cargo test --workspace
```

## Frontend Typechecking
To validate TypeScript compilations in the React workspace:
```bash
npm run typecheck
```

## Quality Audits
To inspect style and security lints:
```bash
npm run lint
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
