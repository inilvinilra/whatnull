# WhatNull

WhatNull is an open-source, Linux-first WhatsApp Web desktop client built with Rust (Tauri 2) and React + TypeScript.

It is not an Electron wrapper. The goal is a lightweight, resource-efficient shell with a strict boundary between the privileged local UI and the remote WhatsApp Web surface.

## Status

WhatNull is at `0.1.0` and is **pre-release**. The table below reflects what is actually implemented, not what is planned.

| Area | State |
| --- | --- |
| Single-window shell, system tray, window state | Working |
| Navigation filtering and external link routing | Working |
| Multi-account profiles with isolated WebKit storage | Working |
| Privacy blur on window unfocus | Working |
| Image and PDF metadata stripping on upload | Working, see limitations |
| Video and audio metadata stripping | Requires `ffmpeg`, currently unreliable |
| Native desktop notifications | Not implemented |
| Voice and video calls | Not implemented |
| Deleted-message preservation | Experimental, opt-in, disabled by default |
| Local logging | Not implemented |

## Design Goals

- **Low resource usage**: minimal memory and CPU footprint.
- **Fast startup**: short launch latency, single native window.
- **Linux integration**: GNOME, KDE, Wayland, XDG paths, native packaging.
- **Webview isolation**: the privileged local shell and the remote WhatsApp surface are separate webviews with separate capability sets.
- **Privacy first**: no telemetry, no analytics, no crash reporting, no intermediary servers.

## Security Boundary

WhatNull runs one native window containing two child webviews. The local `shell` webview holds the application capability set. The remote `whatsapp` webview is granted exactly one narrowly scoped command, `sanitize_upload_files`, so that attachments can be stripped of metadata before they leave the machine. It holds no other local capability.

See [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md) and [docs/WEBVIEW_SECURITY.md](docs/WEBVIEW_SECURITY.md) for details.

## Build Requirements

- Rust stable (1.70+)
- Node.js 18+
- WebKit2GTK and GTK3 development headers
- `ffmpeg` for video and audio metadata stripping (optional)

Debian and Ubuntu:

```bash
sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev
```

Fedora:

```bash
sudo dnf install gtk3-devel webkit2gtk4.1-devel libappindicator-gtk3-devel
```

## Running

Install workspace dependencies from the repository root, then launch the desktop app:

```bash
npm install
npm run tauri dev
```

`npm run dev` starts only the Vite frontend server without the Tauri backend. Use it for isolated UI work; use `npm run tauri dev` to run the actual application.

## Checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
npm run typecheck
npm run lint
```

## Brand Disclaimer

WhatNull is an independent open-source project. It is not affiliated with, endorsed by, sponsored by, or officially connected to WhatsApp or Meta. No official WhatsApp logo or asset is used in this project.

## License

MIT. See [LICENSE](LICENSE).
