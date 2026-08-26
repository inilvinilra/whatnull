# Privacy Policy

WhatNull is built with a privacy-first philosophy. We believe that your private messaging data must remain completely private and under your control.

## No Telemetry & No Analytics

WhatNull does not collect, store, or transmit any telemetry, usage statistics, analytics, or crash reports. There are no tracking scripts, and we do not communicate with any external metrics servers.

## Local Data Storage

All data, session profiles, configuration settings, and window states are stored locally on your device in standard directories (adhering to XDG specifications on Linux). 

- **Messages and Media**: We do not store or log any message bodies, media contents, or attachments on our side. Message rendering is handled entirely within the isolated webview of the official WhatsApp Web.
- **Session Data**: WebView session data (session tokens, cookies, localStorage) is stored securely by the WebKit2GTK engine in isolated storage partitions.
- **Log Files**: Application logs are stored locally for troubleshooting purposes and do not contain any private metadata, phone numbers, contact names, message bodies, or tokens.

## Third-Party Services

When using WhatNull, you connect directly to WhatsApp's official servers. We do not use proxy servers or intermediary backends. Your network traffic is subject to WhatsApp's own privacy policy.
