# WhatNull Architecture

WhatNull is a secure shell around WhatsApp Web, built for low memory and CPU use on Linux.

## Safety Boundary

One native desktop window holds two webviews:

```
┌─────────────────────────────────────────────────────────────┐
│ Native Tauri Window: main                                   │
│ ┌──────────────────────┐  ┌───────────────────────────────┐ │
│ │ shell webview        │  │ whatsapp webview              │ │
│ │ Local React UI       │  │ https://web.whatsapp.com      │ │
│ │ Full capability set  │  │ sanitize_upload_files only    │ │
│ └──────────────────────┘  └───────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

The remote webview holds one narrowly scoped command so attachments can be stripped before upload. It holds no other capability. See [SECURITY_MODEL.md](SECURITY_MODEL.md).

## Runtime Controls

- Navigation filtering keeps WhatsApp-owned surfaces inside the app and routes everything else to the system browser.
- The remote webview is hidden while local modals or the privacy overlay are visible, which avoids native webview stacking problems while keeping a single window.
- WebRTC local candidate filtering is injected into the remote webview as defense in depth. MAC addresses are not exposed to web content by any browser API, so WhatNull does not attempt to modify or spoof them.
- Deleted-message preservation is an optional, disabled-by-default local DOM feature backed by an in-memory cache.
- Metadata stripping for JPEG, PNG, and PDF runs in the privacy crate. Video and audio stripping shells out to `ffmpeg`.

## Project Structure

- `apps/desktop`: React shell and the Tauri binding layer.
- `crates/whatnull_types`: shared enums, structs, event shapes, error categories.
- `crates/whatnull_config`: versioned configuration with atomic file persistence.
- `crates/whatnull_security`: the navigation policy.
- `crates/whatnull_platform`: XDG path resolution, autostart, single-instance socket.
- `crates/whatnull_storage`: per-profile WebView partition directories.
- `crates/whatnull_privacy`: metadata inspection and stripping.
- `crates/whatnull_notification`: notification privacy filtering.
- `crates/whatnull_core`: composition root for the above.
