# WhatNull

WhatNull is an open-source, Linux-first WhatsApp desktop client built with Rust (Tauri 2) and React + TypeScript.

WhatNull is not a bloated Electron wrapper. It is designed to be lightweight, resource-efficient, and highly secure.

## Key Goals
- **Low Resource Usage**: Minimal memory (RAM) and processor (CPU) footprint.
- **Fast Startup**: Optimized system startup and launch latency.
- **Linux Integration**: Strong support for Linux desktop environments (GNOME, KDE, Wayland, XDG, native notifications).
- **Strong Sandbox & Webview Security**: Privileged Local App and Unprivileged WebView isolation.
- **Privacy First**: Zero telemetry, zero tracking, and local-first architecture.

## Brand Disclaimer
WhatNull is an independent open-source project. It is not affiliated with, endorsed by, sponsored by, or officially connected to WhatsApp or Meta. The official WhatsApp logo or assets are not used in this project.

## License
WhatNull is licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Build Requirements
- Rust (stable, 1.70+)
- Node.js (v18+)
- Webkit2GTK (for Linux GUI compilation)

To start the development server:
```bash
npm install
npm run dev
```
