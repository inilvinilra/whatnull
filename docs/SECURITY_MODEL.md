# WhatNull Security Model

This document describes the security controls that are implemented today. Controls that are planned but not yet written are listed at the end so that this file can be trusted as a description of the running code.

## Single-Window WebView Sandbox

The app uses one native Tauri window with two child webviews:

- `shell`: the local React UI. Holds the application capability set defined in `capabilities/default.json`.
- `whatsapp`: the remote `https://web.whatsapp.com` surface.

The two webviews are separate WebKit contexts. The remote surface cannot reach the shell's capability set, cannot read local files, and cannot open arbitrary windows.

## Deliberate IPC Exceptions

The remote `whatsapp` webview is granted exactly two commands. Both capabilities are scoped to the `main` window, the `whatsapp` webview, and the remote origins `web.whatsapp.com`, `*.whatsapp.com`, and `*.whatsapp.net`.

**`sanitize_upload_files`**, through `capabilities/whatsapp-upload-sanitizer.json`. Attachment metadata has to be stripped in the page that owns the file input, before the bytes reach WhatsApp. The command accepts file bytes and returns sanitized bytes. It takes no path argument and performs no filesystem traversal on caller-supplied input.

**`request_shell_action`**, through `capabilities/whatsapp-navbar.json`. The WhatNull navbar is injected into the WhatsApp page, so it needs a way to ask the local shell to open its own interface. The command takes a closed enum with three values: `openSettings`, `openAccounts`, `toggleLock`. Every one of them is a non-destructive interface toggle handled entirely by the local shell. No value carries data from the caller.

The trade-off is explicit: a compromise of `web.whatsapp.com` could call both. For the sanitizer the blast radius is CPU and memory spent on attacker-supplied bytes in a temporary file. For the action channel it is the ability to raise the local shell's own windows, which is an annoyance rather than an escalation. Quitting, session reset, profile creation, deletion and switching, and configuration writes are all unreachable from the remote surface.

## Storage Isolation

Each account profile gets its own WebKit data and cache directory under the XDG data and cache roots. Cookies, local storage, and IndexedDB are partitioned per profile by the WebKit `WebsiteDataManager`.

## Navigation Filtering

`NavigationPolicy` in `crates/whatnull_security` gates every navigation and every new-window request:

- **Rejected schemes**: `file:`, `javascript:`, `data:`, `ftp:`, `ssh:`, `chrome:`, `devtools:`, and anything that is not `http:` or `https:`.
- **Rejected hosts**: `localhost`, loopback and unspecified addresses, IPv4 private and link-local ranges, IPv6 unique-local and link-local ranges. This blocks SSRF and local port scanning from the remote surface.
- **Allowed hosts**: `whatsapp.com`, `whatsapp.net`, `facebook.com`, `fbcdn.net` and their subdomains, which cover WhatsApp Web, its flow and PDF surfaces, and its media CDNs.
- **Internal schemes**: `blob:` and `about:` are allowed because WebKit uses them for WhatsApp's own media and PDF rendering.
- **Everything else** is opened in the system browser rather than loaded in the app.

## Device Permissions

WebKitGTK denies every permission request when no handler is connected, which is why calls did not work at all. WhatNull now answers them explicitly:

- **Microphone and camera**: granted when the matching `permissions` setting is on. Both default to on, because a WhatsApp client that cannot place a call is not doing its job. Both are switchable in Settings.
- **Screen sharing**: refused unless explicitly enabled. WebKit reports display capture through the same request type as the camera, so the two are told apart with `webkit_user_media_permission_is_for_display_device` rather than sharing one switch.
- **Everything else**: geolocation, notifications, pointer lock, device info, media key systems, missing-plugin installation and cross-site data access are refused without asking.

The webview can only ever reach WhatsApp origins, since the navigation policy rejects anything else before it loads, so these grants are scoped by that policy rather than by an origin check in the handler.

## Known Gaps

These are documented rather than claimed as done:

- `blob:` and `about:` are allowed without inspecting the originating origin.
- No Content-Security-Policy is configured for the local shell.
- The user-agent override applied to the remote surface is reversible by page script.
- There is no application logging, so there is nothing to redact yet.
