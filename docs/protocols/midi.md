---
title: MIDI Bridge
description: Bridge MIDI to CLASP
order: 3
---

# MIDI Bridge

Bridge MIDI messages to CLASP signals. Control Change, Note On/Off, Program Change, Pitch Bend, and other MIDI message types are translated into CLASP addresses and values, allowing MIDI controllers and instruments to participate in the CLASP signal graph.

## Quick Start

```bash
clasp midi --target ws://localhost:7330
```

The bridge auto-detects available MIDI ports. To select a specific device:

```bash
clasp midi --device "Launchpad Pro" --target ws://localhost:7330
```

## Address Mapping

MIDI messages are mapped to CLASP addresses using a structured path scheme under the `/midi` namespace:

| MIDI Message | CLASP Address | Example |
|---|---|---|
| Control Change | `/midi/ch/{channel}/cc/{controller}` | `/midi/ch/1/cc/74` |
| Note On / Off | `/midi/ch/{channel}/note/{note}` | `/midi/ch/1/note/60` |
| Program Change | `/midi/ch/{channel}/program` | `/midi/ch/10/program` |
| Pitch Bend | `/midi/ch/{channel}/pitch` | `/midi/ch/1/pitch` |
| Channel Pressure | `/midi/ch/{channel}/pressure` | `/midi/ch/1/pressure` |
| Poly Aftertouch | `/midi/ch/{channel}/poly/{note}` | `/midi/ch/1/poly/60` |

Channels are numbered 1-16 (matching MIDI convention). Note numbers are 0-127.

The mapping is bidirectional: setting a value on `/midi/ch/1/cc/74` sends a CC message on channel 1, controller 74, to the connected MIDI device.

## Signal Type Mapping

| MIDI Message | CLASP Signal Type | Details |
|---|---|---|
| Control Change | Param | Continuous value, persisted in state |
| Note On | Event | Emitted with velocity value |
| Note Off | Event | Emitted with velocity 0 |
| Program Change | Param | Program number as integer |
| Pitch Bend | Param | Centered at 0.5, range 0.0-1.0 |
| Channel Pressure | Param | Pressure value 0.0-1.0 |

## Value Mapping

MIDI values (0-127) are normalized to floating-point range 0.0-1.0 for CC and pressure messages:

| MIDI Value | CLASP Value |
|---|---|
| 0 | 0.0 |
| 64 | 0.5 (approx) |
| 127 | 1.0 |

This normalization makes MIDI values directly compatible with other CLASP sources. A fader sending MIDI CC and an OSC fader sending 0.0-1.0 produce equivalent values in the CLASP state.

Note On events carry the velocity as a normalized float (0.0-1.0). Note Off events carry velocity 0.

Pitch Bend is normalized from the 14-bit MIDI range (0-16383) to 0.0-1.0, with center at 0.5.

Program Change values remain as integers (0-127).

## Device Selection

List available MIDI devices:

```bash
clasp midi --list
```

Output:

```
Available MIDI ports:
  [0] Launchpad Pro  (input/output)
  [1] IAC Driver     (input/output)
  [2] USB MIDI       (input only)
```

Select by name or index:

```bash
clasp midi --device "Launchpad Pro" --target ws://localhost:7330
clasp midi --device 0 --target ws://localhost:7330
```

To bridge multiple MIDI devices, run multiple bridge instances with different namespaces:

```bash
clasp midi --device "Launchpad Pro" --namespace /midi/launchpad --target ws://localhost:7330
clasp midi --device "USB MIDI" --namespace /midi/keyboard --target ws://localhost:7330
```

## Troubleshooting

**No MIDI devices found**
- Check that the device is connected and recognized by the OS
- On Linux, ensure the user has permission to access `/dev/snd/*` or is in the `audio` group
- On macOS, virtual MIDI ports (IAC Driver) must be enabled in Audio MIDI Setup

**Messages received but values wrong**
- CC values are normalized to 0.0-1.0. If you need raw 0-127 integers, use a value transform in the bridge configuration
- Note numbers are 0-indexed (Middle C = 60)

**Bidirectional feedback loop**
- If a MIDI controller sends a CC and the bridge echoes it back, some controllers enter a feedback loop. Use the `--no-echo` flag to suppress outbound messages that originated from the same device

**High latency**
- MIDI is inherently low-latency. If you observe delays, check the network connection between the bridge and the router
- Use `--log-level debug` to see per-message timestamps

## Next Steps

- [MQTT Bridge](mqtt.md) -- bridge MQTT topics and messages
- [Protocol Bridges Overview](../protocols/README.md) -- all 8 protocol bridges
- Bridge Configuration -- custom address mappings and value transforms
