# WhatNull Security Model

This document describes the security controls that are implemented today. Controls that are planned but not yet written are listed at the end so that this file can be trusted as a description of the running code.

## Single-Window WebView Sandbox

The app uses one native Tauri window with two child webviews:

- `shell`: the local React UI. Holds the application capability set defined in `capabilities/default.json`.
- `whatsapp`: the remote `https://web.whatsapp.com` surface.

The two webviews are separate WebKit contexts. The remote surface cannot reach the shell's capability set, cannot read local files, and cannot open arbitrary windows.

## The One Deliberate IPC Exception

The remote `whatsapp` webview is granted exactly one command, `sanitize_upload_files`, through `capabilities/whatsapp-upload-sanitizer.json`. This capability is scoped to the `main` window, the `whatsapp` webview, and the remote origins `web.whatsapp.com`, `*.whatsapp.com`, and `*.whatsapp.net`.

This exists because attachment metadata must be stripped in the page that owns the file input, before the bytes reach WhatsApp. The command accepts file bytes and returns sanitized bytes. It takes no path argument and performs no filesystem traversal on caller-supplied paths.

The trade-off is explicit: a compromise of `web.whatsapp.com` could call this command. The blast radius is bounded to CPU and memory spent sanitizing attacker-supplied bytes in a temporary file. No other command is reachable from the remote surface.

## Storage Isolation

Each account profile gets its own WebKit data and cache directory under the XDG data and cache roots. Cookies, local storage, and IndexedDB are partitioned per profile by the WebKit `WebsiteDataManager`.

## Navigation Filtering

`NavigationPolicy` in `crates/whatnull_security` gates every navigation and every new-window request:

- **Rejected schemes**: `file:`, `javascript:`, `data:`, `ftp:`, `ssh:`, `chrome:`, `devtools:`, and anything that is not `http:` or `https:`.
- **Rejected hosts**: `localhost`, loopback and unspecified addresses, IPv4 private and link-local ranges, IPv6 unique-local and link-local ranges. This blocks SSRF and local port scanning from the remote surface.
- **Allowed hosts**: `whatsapp.com`, `whatsapp.net`, `facebook.com`, `fbcdn.net` and their subdomains, which cover WhatsApp Web, its flow and PDF surfaces, and its media CDNs.
- **Internal schemes**: `blob:` and `about:` are allowed because WebKit uses them for WhatsApp's own media and PDF rendering.
- **Everything else** is opened in the system browser rather than loaded in the app.

## Known Gaps

These are documented rather than claimed as done:

- `blob:` and `about:` are allowed without inspecting the originating origin.
- No Content-Security-Policy is configured for the local shell.
- The user-agent override applied to the remote surface is reversible by page script.
- There is no application logging, so there is nothing to redact yet.
