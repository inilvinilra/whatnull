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
│ │ Full capability set  │  │ two scoped commands           │ │
│ │ Shown on demand      │  │ Injected WhatNull navbar      │ │
│ └──────────────────────┘  └───────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

Both webviews fill the whole window and only one is visible at a time. WhatsApp is shown during normal use; the shell is raised over it for onboarding, the privacy lock, settings and the account switcher. GTK gives sibling webviews no usable stacking order, so visibility is switched explicitly rather than relying on z-order.

The WhatNull navbar is injected into the WhatsApp page inside a closed shadow root, which is why the shell does not need a permanent strip of the window. The remote webview holds two narrowly scoped commands, one to strip attachment metadata and one to ask the shell to open its own interface. See [SECURITY_MODEL.md](SECURITY_MODEL.md).

## Runtime Controls

- Navigation filtering keeps WhatsApp-owned surfaces inside the app and routes everything else to the system browser.
- The remote webview is hidden while local modals or the privacy overlay are visible, which avoids native webview stacking problems while keeping a single window.
- Both webviews are resized together from one place, so a window resize, a scale-factor change and a restored window position cannot leave one of them at stale bounds.
- Voice and video calls require WebKitGTK's media settings, which are off by default and which wry does not set, plus a `permission-request` handler, without which WebKit refuses `getUserMedia` outright. WhatNull enables media stream, WebRTC, MediaSource and encrypted media on the remote webview and answers permission requests from the `permissions` section of the config. Camera and microphone default to allowed, screen sharing to refused, and every other permission class is refused outright.
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
