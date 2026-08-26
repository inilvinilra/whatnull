#!/usr/bin/env bash
set -euo pipefail

echo "=========================================="
echo " Building WhatNull Open Source Release"
echo "=========================================="

echo "[1/3] Running Cargo Unit Tests..."
cargo test -p whatnull_types -p whatnull_platform -p whatnull_config -p whatnull_security -p whatnull_storage -p whatnull_core -p whatnull_notification

echo "[2/3] Building React + TypeScript Frontend Bundle..."
npm run build

echo "[3/3] Package configuration ready for AppImage, DEB, and RPM bundles."
echo "To compile distribution binaries on a Linux host with WebKit2GTK installed:"
echo "  npx tauri build"
echo "=========================================="
