# WebView Security Guidelines

This document details the security constraints applied to WebViews inside WhatNull.

## Isolated Contexts
WhatNull runs as a single native window with separate child webviews. The local React shell owns privileged controls, while the WhatsApp webview owns only the remote web session.

WhatsApp WebView instances run in separate execution profiles:
- Each profile partition gets its own isolated cookie and storage bucket.
- Cross-account session leaks are impossible due to directory-level partition isolation.

## Security Policies
- **Disabled File Access**: WebViews cannot load local `file://` resources.
- **IPC Restrictions**: Only the local `shell` webview is listed in the app capability file. The remote `whatsapp` webview is not granted local Tauri commands.
- **Custom Context Menu**: Developer tools and inspect elements are disabled in production builds.
- **Scheme Filters**: Blocklist checks are run synchronously on every navigation request before load.
- **Internal WhatsApp Tools**: WhatsApp PDF/tooling hosts such as `flows.whatsapp.net` and `webtp.whatsapp.net` stay inside the WhatNull webview instead of opening an external browser.
