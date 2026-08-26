# Privacy Architecture

WhatNull is designed to be privacy-respecting and audit-friendly.

## Principles
- **No Intermediary Servers**: All traffic flows directly from the client to official WhatsApp endpoints.
- **Zero Metrics Collection**: No analytics trackers, telemetry trackers, or crash upload clients are integrated into the application.
- **Local Settings**: App configuration and window size parameters are written to standard local files and never synced online.

## Log Sanitization
Log directories only store safe technical lifecycle events (e.g. `app.started`, `webview.created`, `webview.failed`). Sensitive information, including cookies, session tokens, message text, phone numbers, contact names, and attachment paths, is redacted before printing to stdout or disk.
