# WebView Security Guidelines

This document details the security constraints applied to WebViews inside WhatNull.

## Isolated Contexts
WhatsApp WebView instances run in completely separate execution profiles:
- Each profile partition gets its own isolated cookie and storage bucket.
- Cross-account session leaks are impossible due to directory-level partition isolation.

## Security Policies
- **Disabled File Access**: WebViews cannot load local `file://` resources.
- **IPC Restrictions**: WebViews are blocked from invoking any Tauri IPC calls.
- **Custom Context Menu**: Developer tools and inspect elements are disabled in production builds.
- **Scheme Filters**: Blocklist checks are run synchronously on every navigation request before load.
