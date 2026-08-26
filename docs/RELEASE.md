# Release Procedure

This document describes how releases are built, signed, and published.

## Release Process
1. Verify CI is green on pull request.
2. Increment package and binary version strings.
3. Update CHANGELOG.md.
4. Run full production build checklist:
   - Lint check.
   - Rust test workspace.
   - Frontend build checks.
5. Create a tagged commit (e.g. `v0.1.0`).
6. Build release artifacts:
   ```bash
   npx tauri build
   ```
7. Sign bundle files using code-signing signatures.
8. Upload generated Debian, RPM, and AppImage artifacts to releases.
