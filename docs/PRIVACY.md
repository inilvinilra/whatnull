# Privacy Architecture

WhatNull is designed to be privacy-respecting and audit-friendly.

## Principles
- **No Intermediary Servers**: All traffic flows directly from the client to official WhatsApp endpoints.
- **Zero Metrics Collection**: No analytics trackers, telemetry trackers, or crash upload clients are integrated into the application.
- **Local Settings**: App configuration and window size parameters are written to standard local files and never synced online.

## Logging

WhatNull does not currently write application logs. Local logging is planned. When it lands it will stay on your device, will never contain message bodies, contact names, phone numbers, or tokens, and will be clearable from the interface.
