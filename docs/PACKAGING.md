# Linux Packaging Specifications

WhatNull targets multiple Linux native distribution formats.

## Bundle Targets
- **AppImage**: Self-contained executable package running across major distributions.
- **Debian Package (.deb)**: Target format for Debian, Ubuntu, Linux Mint, and derivatives.
- **RPM Package (.rpm)**: Target format for Fedora, RHEL, CentOS, and openSUSE.

## Sandbox Integration
- **Flatpak**: Planned release format with sandboxed permissions restricting camera, microphone, and filesystem access to the bare minimum.
- **Arch AUR**: Binary package recipe (`whatnull-bin`) to download and unpack releases under `/usr/bin`.
