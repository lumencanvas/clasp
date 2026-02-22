---
title: "Integrations"
description: "Connect CLASP to popular creative software and platforms."
section: integrations
order: 0
---
# Integrations

Connect CLASP to popular creative software and platforms.

## Creative Software

### [TouchOSC](touchosc.md)
Mobile OSC controller for iOS and Android.

### [Resolume](resolume.md)
VJ software for live video performance.

### [QLab](qlab.md)
Show control software for theater and live events.

### [Ableton Live](ableton.md)
Digital audio workstation for music production.

### [TouchDesigner](touchdesigner.md)
Visual programming for interactive media.

### [MadMapper](madmapper.md)
Projection mapping software.

## Home Automation

### [Home Assistant](home-assistant.md)
Open-source home automation platform.

## Integration Patterns

| Software | Protocol | CLASP Bridge |
|----------|----------|--------------|
| TouchOSC | OSC | `clasp osc` |
| Resolume | OSC | `clasp osc` |
| QLab | OSC | `clasp osc` |
| Ableton | MIDI, OSC | `clasp midi`, `clasp osc` |
| TouchDesigner | OSC, MIDI | `clasp osc`, `clasp midi` |
| MadMapper | OSC | `clasp osc` |
| Home Assistant | MQTT | `clasp mqtt` |

## Quick Start

1. Start CLASP router: `clasp server`
2. Start protocol bridge: `clasp osc --port 9000`
3. Configure software to send/receive on bridge port
4. Connect your CLASP client

## Need Another Integration?

Check if your software supports:
- **OSC** → Use `clasp osc`
- **MIDI** → Use `clasp midi`
- **MQTT** → Use `clasp mqtt`
- **HTTP/REST** → Use `clasp http`
- **WebSocket** → Connect directly or use bridge

File a feature request on [GitHub](https://github.com/lumencanvas/clasp/issues) for new integrations.
