# Changelog

All notable changes to this project are documented in this file.

## [Unreleased]
### Fixed
- The window no longer shows a strip of the local shell above the WhatsApp surface. Both webviews now fill the window and visibility is switched explicitly, because GTK gives sibling webviews no usable stacking order.
- Startup failures print the reason to stderr instead of exiting silently with status 1.
- IPv6 addresses were never rejected by the navigation policy. `Url::host_str` returns IPv6 literals wrapped in brackets, so they never parsed as an IP and the entire IPv6 private-address branch was dead code. Host inspection now uses the parsed `url::Host`, and IPv4-mapped IPv6 addresses such as `::ffff:127.0.0.1` are covered too.
- The account switcher now surfaces command errors instead of writing them to the console, so refusals like "Cannot delete the active profile" are visible.

### Added
- A WhatNull navbar injected into the WhatsApp page inside a closed shadow root, carrying the privacy lock, account profiles and settings actions.
- `request_shell_action`, a closed three-value command letting that navbar ask the local shell to open its own interface.
- Attachment metadata sanitization for the remote WhatsApp surface, backed by a new `sanitize_upload_files` command.
- A dedicated, narrowly scoped capability granting that single command to the `whatsapp` webview.
- Explicit Tauri app manifest listing every exposed command, so the generated ACL matches the invoke handler.

### Changed
- `set_whatsapp_visible` and `set_shell_overlay_mode` are replaced by a single `set_overlay_visible`. The frontend previously issued the two as separate calls that could interleave.
- The permanent 60px shell sidebar is gone, along with the `Sidebar` component and the unused `App.css`.
- Source comments are removed in favour of named constants and helper functions, matching the contribution rules. Marker and chunk magic numbers in the privacy crate are now named.
- The account switcher derives its profile list from the config store instead of issuing a second `list_profiles` call and holding a duplicate copy in local state.
- The JPEG stripping test asserted only that the call returned either variant. It now builds a JPEG carrying an EXIF segment and asserts the segment is gone and the JFIF segment survives.
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
