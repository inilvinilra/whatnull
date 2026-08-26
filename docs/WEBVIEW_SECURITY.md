# WebView Security

This document details the constraints applied to the two webviews inside WhatNull.

## Contexts

WhatNull runs a single native window with two child webviews. The local `shell` webview owns the privileged controls. The remote `whatsapp` webview owns the WhatsApp Web session and nothing else.

Each account profile gets its own WebKit data directory, so cookies and storage are partitioned per profile at the `WebsiteDataManager` level.

## Capability Assignment

- `capabilities/default.json` binds the application command set to the `shell` webview only.
- `capabilities/whatsapp-upload-sanitizer.json` binds exactly one command, `sanitize_upload_files`, to the `whatsapp` webview, scoped to WhatsApp origins. See [SECURITY_MODEL.md](SECURITY_MODEL.md) for the reasoning and the accepted trade-off.

No other command is reachable from the remote surface.

## Navigation Handling

Scheme and host checks run synchronously on every navigation request before the load starts, and again on every new-window request. New windows are always denied; the request is either redirected inside the existing webview or handed to the system browser, depending on the policy decision.

WhatsApp's own tooling hosts, such as `flows.whatsapp.net` and `webtp.whatsapp.net`, stay inside WhatNull instead of opening an external browser, so the PDF viewer and flow surfaces work.

## Injected Script

WhatNull injects a script into the remote surface for user-agent normalization, WebRTC local-candidate filtering, upload sanitization, and the optional deleted-message feature. The script runs in the page's own world, which means page script can observe and override it. It is defense in depth, not a security boundary.

## Known Gaps

- Developer tools are not explicitly disabled in release builds; the `enable_dev_tools` setting is stored but not yet applied.
- The injected script exposes a `window.__WHATNULL_STATUS__` object that page script can read.
