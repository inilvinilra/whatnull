# Changelog

All notable changes to this project are documented in this file.

## [Unreleased]
### Added
- Attachment metadata sanitization for the remote WhatsApp surface, backed by a new `sanitize_upload_files` command.
- A dedicated, narrowly scoped capability granting that single command to the `whatsapp` webview.
- Explicit Tauri app manifest listing every exposed command, so the generated ACL matches the invoke handler.

### Changed
- Window bounds are now synchronized after restoring a saved window position.
- Documentation now describes the implemented behaviour only. The security docs previously claimed the remote webview held no IPC capability, which stopped being true once upload sanitization landed. Known gaps are listed explicitly in `docs/SECURITY_MODEL.md` and `docs/WEBVIEW_SECURITY.md`.
- `PRIVACY.md` no longer describes a log redaction pipeline. No application logging exists yet.
- `README.md` now carries an honest feature status table and the correct command for launching the app.

## [0.1.0] - 2026-08-26
### Added
- Monorepo structure setup.
- React + TypeScript + Vite frontend shell.
- Tauri v2 window and system tray support.
- Config manager with atomic writes.
- NavigationPolicy rules engine.
- Single-instance listener using Unix Domain Sockets.
- Linux XDG paths resolver.
