# Development Environment Setup

This document describes how to set up the development environment for WhatNull.

## Pre-requisites
- Rust stable and Cargo package manager.
- Node.js (v18+) and npm.
- Linux build dependencies:
  - Debian/Ubuntu: `sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev`
  - Fedora: `sudo dnf install gtk3-devel webkit2gtk4.1-devel libappindicator-gtk3-devel`

## Running Local Dev Server
Install workspace dependencies and run tauri dev:
```bash
npm install
npm run dev
```

To run tauri development bindings directly:
```bash
npx tauri dev
```
