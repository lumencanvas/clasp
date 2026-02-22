---
title: "Install CLI"
description: "Install the CLASP command-line tool for running routers and bridges."
section: how-to
order: 1
---
# Install CLI

Install the CLASP command-line tool for running routers and bridges.

## Using Cargo (Recommended)

```bash
cargo install clasp-cli
```

**Requirements:**
- Rust 1.75 or later

Verify installation:
```bash
clasp --version
```

## From Source

```bash
git clone https://github.com/lumencanvas/clasp.git
cd clasp
cargo install --path crates/clasp-cli
```

## Platform-Specific Notes

### macOS

No additional dependencies needed.

### Linux

Install system dependencies first:

```bash
# Ubuntu/Debian
sudo apt install libasound2-dev libudev-dev

# Fedora
sudo dnf install alsa-lib-devel systemd-devel

# Arch
sudo pacman -S alsa-lib systemd
```

### Windows

No additional dependencies needed. Use PowerShell or CMD:

```powershell
cargo install clasp-cli
```

## Verify Installation

```bash
# Show help
clasp --help

# Start a router
clasp server --port 7330

# Start an OSC bridge
clasp osc --port 9000
```

## Available Commands

| Command | Description |
|---------|-------------|
| `clasp server` | Start a CLASP router |
| `clasp osc` | Start an OSC bridge |
| `clasp midi` | Start a MIDI bridge |
| `clasp mqtt` | Start an MQTT bridge |
| `clasp http` | Start an HTTP REST bridge |
| `clasp artnet` | Start an Art-Net bridge |

## Update

```bash
cargo install clasp-cli --force
```

## Uninstall

```bash
cargo uninstall clasp-cli
```

## Next Steps

- [Start a Router](../connections/start-router.md)
- [Add Protocol Bridges](../connections/add-osc.md)
