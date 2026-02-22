---
title: "Install Desktop App"
description: "Install the CLASP Bridge desktop application for visual configuration."
section: how-to
order: 6
---
# Install Desktop App

Install the CLASP Bridge desktop application for visual configuration.

## Download

Download the latest release for your platform:

- **macOS:** [CLASP Bridge.dmg](https://github.com/lumencanvas/clasp/releases/latest)
- **Windows:** [CLASP Bridge Setup.exe](https://github.com/lumencanvas/clasp/releases/latest)
- **Linux:** [clasp-bridge.AppImage](https://github.com/lumencanvas/clasp/releases/latest)

## Installation

### macOS

1. Open `CLASP Bridge.dmg`
2. Drag CLASP Bridge to Applications
3. On first launch, right-click and select "Open" (Gatekeeper)

### Windows

1. Run `CLASP Bridge Setup.exe`
2. Follow the installer prompts
3. Launch from Start Menu

### Linux

```bash
chmod +x clasp-bridge.AppImage
./clasp-bridge.AppImage
```

Or install system-wide:
```bash
sudo mv clasp-bridge.AppImage /usr/local/bin/clasp-bridge
```

## Features

The desktop app provides:

- **Visual Protocol Management** — Add and configure bridges
- **Signal Monitoring** — Watch messages in real-time
- **Learn Mode** — Auto-capture addresses from incoming signals
- **REST API Builder** — Create HTTP endpoints
- **Embedded Router** — No separate server needed

## First Launch

1. The app starts an embedded CLASP router automatically
2. Click "Add Protocol" to add OSC, MIDI, etc.
3. Configure the protocol settings
4. Click "Start" to begin bridging

## System Requirements

| Platform | Requirement |
|----------|-------------|
| macOS | 10.15 (Catalina) or later |
| Windows | Windows 10 or later |
| Linux | Ubuntu 20.04+ or equivalent |

## Troubleshooting

### macOS: "App is damaged"

```bash
xattr -d com.apple.quarantine /Applications/CLASP\ Bridge.app
```

### Linux: Missing libraries

```bash
# Ubuntu/Debian
sudo apt install libgtk-3-0 libwebkit2gtk-4.0

# Fedora
sudo dnf install gtk3 webkit2gtk3
```

### Windows: SmartScreen warning

Click "More info" → "Run anyway"

## Next Steps

- [Add OSC Connection](../connections/add-osc.md)
- [Add MIDI Connection](../connections/add-midi.md)
