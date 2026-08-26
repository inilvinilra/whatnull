# WhatNull Architectural Specification

WhatNull implements a secure, modular wrapper for WhatsApp Web, optimizing memory, CPU, and sandboxing.

## Safety Boundary

WhatNull divides the application into two strict security domains:

```
┌─────────────────────────────────┐
│    Privileged Local App         │ (Rust Core + Local React UI)
└────────────────┬────────────────┘
                 │
           Security Wall (Restricted IPC)
                 │
┌────────────────▼────────────────┐
│   Unprivileged WhatsApp Web     │ (Official WebView Instance)
└─────────────────────────────────┘
```

The Remote WebView cannot invoke privileged Tauri commands. Only the local React UI (Settings, Accounts, About) has access to the restricted Tauri command set.

## Project Structure

- `apps/desktop`: The frontend React shell and Tauri binding wrapper.
- `crates/whatnull_types`: Core enums, structures, event formats, and error categories.
- `crates/whatnull_config`: Configuration layout with versioning and atomic file persistence.
- `crates/whatnull_security`: Navigation filter enforcing scheme validations and private IP blocks.
- `crates/whatnull_platform`: Linux-first paths manager and single-instance listener.
- `crates/whatnull_storage`: WebView partition directories and keyring wrapper stubs.
- `crates/whatnull_core`: Orchestrates multi-module business logic.
