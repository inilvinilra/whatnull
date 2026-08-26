# WhatNull Security Model

WhatNull is engineered with security-first design principles. This document outlines the security controls implemented to protect your data and system.

## WebView Sandbox

The remote WhatsApp WebView is isolated inside a separate container:
- **No IPC Access**: The remote WebView is completely restricted from calling any Tauri command.
- **No Local Filesystem Access**: Safe directory sandboxing is enforced.
- **Separate Cache Partitions**: Multi-account profile directories prevent cross-account cookie leakage.

## Navigation Filtering

Our `NavigationPolicy` acts as an egress gatekeeper for the WebView:
- **Blocked Schemes**: Schemas like `file:`, `javascript:`, `data:`, `blob:`, `ftp:`, `ssh:`, `about:`, `chrome:`, and `devtools:` are rejected.
- **Local Address Blocking**: Addresses targeting `localhost`, loopback IPs (`127.0.0.1`, `::1`), or private subnets are blocked to prevent SSRF and local port scanning.
- **Allowed Hosts**: Only `web.whatsapp.com` and its subdomains are allowed to load. All other safe HTTPS links are routed out of WhatNull and opened in the system's default browser.
