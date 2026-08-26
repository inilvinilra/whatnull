# WhatNull Security Model

WhatNull is engineered with security-first design principles. This document outlines the security controls implemented to protect your data and system.

## Single-Window WebView Sandbox

The app uses one native Tauri window with separate child webviews:
- `shell`: local React UI with the restricted capability set.
- `whatsapp`: remote `https://web.whatsapp.com` surface with no local IPC capability.

The remote WhatsApp WebView is isolated inside its own child webview:
- **No IPC Access**: The remote WebView is not listed in any capability and cannot call Tauri commands.
- **No Local Filesystem Access**: Safe directory sandboxing is enforced.
- **Separate Cache Partitions**: Multi-account profile directories prevent cross-account cookie leakage.

## Navigation Filtering

Our `NavigationPolicy` acts as an egress gatekeeper for the WebView:
- **Blocked Schemes**: Schemas like `file:`, `javascript:`, `data:`, `ftp:`, `ssh:`, `chrome:`, and `devtools:` are rejected.
- **Internal Schemes**: `blob:` and `about:` are allowed for WhatsApp-owned media/PDF flows that WebKit uses internally.
- **Local Address Blocking**: Addresses targeting `localhost`, loopback IPs (`127.0.0.1`, `::1`), or private subnets are blocked to prevent SSRF and local port scanning.
- **Allowed Hosts**: WhatsApp-owned surfaces and required media/CDN hosts are allowed to load. Other safe HTTP(S) links are routed out of WhatNull and opened in the system's default browser.
