# Linux Desktop Support

WhatNull treats Linux as a first-class citizen. Features are optimized for Linux desktop environments first.

## Supported Desktop Environments
- **GNOME**
- **KDE Plasma**
- **Hyprland & Sway** (Wayland tiling managers)
- **XFCE & Cinnamon**

## Wayland & X11 Compliance
Wayland is natively supported. Graphic acceleration settings and window decorations are tuned to run smoothly without falling back to X11 emulation.

## XDG Directories Integration
WhatNull strictly follows XDG specifications for file paths:
- **Config**: `$XDG_CONFIG_HOME/whatnull` (defaults to `~/.config/whatnull`)
- **Data**: `$XDG_DATA_HOME/whatnull` (defaults to `~/.local/share/whatnull`)
- **Cache**: `$XDG_CACHE_HOME/whatnull` (defaults to `~/.cache/whatnull`)
- **State**: `$XDG_STATE_HOME/whatnull` (defaults to `~/.local/state/whatnull`)
- **Downloads**: Complies with path defined in `~/.config/user-dirs.dirs`.

## System Autostart
Enabling autostart writes a `.desktop` launcher to `~/.config/autostart/whatnull.desktop`, complying with standard Linux boot startup specifications.
