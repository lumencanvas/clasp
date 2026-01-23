# Protocol Overview

CLASP (Creative Low-Latency Application Streaming Protocol) is a modern protocol designed for real-time creative applications. This document describes the design principles and high-level architecture.

## Design Principles

### 1. Transport-Agnostic

CLASP is transport-agnostic by design. The protocol works over any byte transport:
- WebSocket (recommended baseline)
- WebRTC DataChannel (low-latency P2P)
- QUIC (native apps, mobile)
- UDP (LAN devices, embedded)
- Serial/UART (hardware)
- Bluetooth Low Energy (IoT)

The frame format makes no assumptions about packet ordering, reliability, connection semantics, or MTU size.

### 2. Progressive Enhancement

Simple things are simple:
```
Hello → Send message → Done (3 lines of code)
```

Complex features (encryption, discovery, state sync) are opt-in.

### 3. Semantic Signals

The protocol knows the difference between signal types:
- **Param** — A stateful value (fader position)
- **Event** — An ephemeral trigger (button press)
- **Stream** — High-rate continuous data (motion)
- **Gesture** — Phased input (touch start/move/end)
- **Timeline** — Time-indexed automation

### 4. Discovery Is First-Class

Finding devices doesn't require configuration. Automatic discovery via:
- mDNS/DNS-SD on LAN
- UDP broadcast fallback
- Cloud rendezvous for WAN

Manual configuration always possible as fallback.

### 5. State Is Truth

Parameters have authoritative values with revision numbers. "What is the current brightness?" always has a definitive answer.

### 6. Timing Is Deterministic

- Bundles can be scheduled for future execution
- Clock synchronization is built-in
- Jitter buffers for WiFi/WAN

### 7. Security Without Ceremony

- Encrypted by default in production
- Local development doesn't require certificates
- Capability tokens for fine-grained access control

### 8. Legacy Is Respected

Bridges for MIDI, OSC, DMX are defined in the spec, not afterthoughts.

## Non-Goals

- **Not a media transport**: CLASP carries control signals, not audio/video streams
- **Not a file format**: Show files and presets are application-level concerns
- **Not a UI specification**: How controls are displayed is up to applications

## Protocol Version

Current version: **1.0**

The version is communicated during the HELLO/WELCOME handshake.

## Conformance Levels

| Level | Requirements | Target |
|-------|--------------|--------|
| **Minimal** | WebSocket, HELLO/WELCOME, SET/PUBLISH | Browser apps |
| **Standard** | Minimal + SUBSCRIBE, Param/Event/Stream | Desktop apps |
| **Full** | Standard + Timeline, Gestures, Discovery, Bridges | Professional tools |
| **Embedded** | UDP, numeric addresses, fixed types | Microcontrollers |

### Minimal Implementation (~200 LOC)

A minimal CLASP client needs:
1. WebSocket connection
2. Binary or MessagePack encode/decode
3. HELLO/WELCOME handshake
4. SET for sending params
5. PUBLISH for receiving

## Related Documents

- [Messages](messages.md) — Complete message catalog
- [Signal Types](signal-types.md) — Param, Event, Stream, Gesture, Timeline
- [Addressing](addressing.md) — Address format and wildcards
- [Data Types](data-types.md) — Value encoding
- [Frame Format](frame-format.md) — Binary wire format
- [QoS Levels](qos.md) — Reliability guarantees
