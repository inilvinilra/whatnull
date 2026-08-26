# WhatNull Architectural Specification

WhatNull implements a secure, modular wrapper for WhatsApp Web, optimizing memory, CPU, and sandboxing.

## Safety Boundary

WhatNull uses one native desktop window with two webviews inside it:

```
┌─────────────────────────────────────────────────────────────┐
│ Native Tauri Window: main                                   │
│ ┌──────────────────────┐  ┌───────────────────────────────┐ │
│ │ shell webview        │  │ whatsapp webview              │ │
│ │ Local React UI       │  │ https://web.whatsapp.com      │ │
│ │ Privileged IPC       │  │ No local IPC capability       │ │
│ └──────────────────────┘  └───────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

The remote WhatsApp webview cannot invoke privileged Tauri commands because capabilities are assigned only to the local `shell` webview. The application still feels like a single full-screen app window to the user.

## Runtime Controls

- Navigation filtering keeps WhatsApp-owned surfaces such as `web.whatsapp.com`, `flows.whatsapp.net`, `webtp.whatsapp.net`, and required media/CDN hosts inside WhatNull.
- Non-WhatsApp external links are opened in the system browser.
- The WhatsApp child webview is temporarily hidden while local modals or the privacy overlay are visible, avoiding native-webview stacking problems while preserving a single window.
- WebRTC local IP candidate filtering and device surface normalization are injected into the remote webview as defense-in-depth. MAC addresses are not exposed to normal web content, so WhatNull does not modify host network adapter addresses.
- Deleted-message preservation is a local DOM recovery feature backed by an in-memory cache in the WhatsApp webview.
- JPEG, PNG, PDF, video, and audio metadata stripping is implemented in the privacy crate. Video/audio stripping requires `ffmpeg`.

## Project Structure

- `apps/desktop`: The frontend React shell and Tauri binding wrapper.
- `crates/whatnull_types`: Core enums, structures, event formats, and error categories.
- `crates/whatnull_config`: Configuration layout with versioning and atomic file persistence.
- `crates/whatnull_security`: Navigation filter enforcing scheme validations and private IP blocks.
- `crates/whatnull_platform`: Linux-first paths manager and single-instance listener.
- `crates/whatnull_storage`: WebView partition directories and keyring wrapper stubs.
- `crates/whatnull_core`: Orchestrates multi-module business logic.
