---
title: "Reference Documentation"
description: "Complete, accurate technical documentation for CLASP."
section: reference
order: 0
---
# Reference Documentation

Complete, accurate technical documentation for CLASP.

## Protocol Specification

Core protocol details for implementers:

- [Protocol Overview](protocol/overview.md) — Design principles and architecture
- [Messages](protocol/messages.md) — All message types (HELLO, SET, SUBSCRIBE, etc.)
- [Signal Types](protocol/signal-types.md) — Param, Event, Stream, Gesture, Timeline
- [Addressing](protocol/addressing.md) — Address format and wildcard patterns
- [Data Types](protocol/data-types.md) — Value types and encoding
- [Frame Format](protocol/frame-format.md) — Binary frame structure
- [QoS Levels](protocol/qos.md) — Fire, Confirm, Commit reliability

## API Reference

Library documentation by language:

### Rust
- [clasp-core](api/rust/clasp-core.md) — Core types and codec
- [clasp-client](api/rust/clasp-client.md) — Client library
- [clasp-router](api/rust/clasp-router.md) — Router library
- [clasp-bridge](api/rust/clasp-bridge.md) — Bridge implementations
- [clasp-transport](api/rust/clasp-transport.md) — Transport implementations
- [clasp-discovery](api/rust/clasp-discovery.md) — Discovery mechanisms
- [clasp-embedded](api/rust/clasp-embedded.md) — Embedded/no_std client

### JavaScript
- [@clasp-to/core](api/javascript/clasp-core.md) — JavaScript/TypeScript client
- [Browser Usage](api/javascript/browser.md) — Browser-specific notes
- [Node.js Usage](api/javascript/nodejs.md) — Node.js-specific notes

### Python
- [clasp-to](api/python/clasp-to.md) — Python client package

## CLI Reference

Command-line tool documentation:

- [clasp server](cli/clasp-server.md) — Start a CLASP router
- [clasp osc](cli/clasp-osc.md) — OSC protocol connection
- [clasp midi](cli/clasp-midi.md) — MIDI protocol connection
- [clasp mqtt](cli/clasp-mqtt.md) — MQTT protocol connection
- [clasp http](cli/clasp-http.md) — HTTP REST API

## Bridge Reference

Protocol-to-CLASP mapping documentation:

- [OSC Bridge](bridges/osc.md) — OSC ↔ CLASP mapping
- [MIDI Bridge](bridges/midi.md) — MIDI ↔ CLASP mapping
- [Art-Net Bridge](bridges/artnet.md) — Art-Net ↔ CLASP mapping
- [DMX Bridge](bridges/dmx.md) — DMX ↔ CLASP mapping
- [MQTT Bridge](bridges/mqtt.md) — MQTT ↔ CLASP mapping
- [sACN Bridge](bridges/sacn.md) — sACN ↔ CLASP mapping
- [HTTP Bridge](bridges/http.md) — HTTP ↔ CLASP mapping

## Transport Reference

Network transport documentation:

- [WebSocket](transports/websocket.md) — WebSocket transport
- [QUIC](transports/quic.md) — QUIC transport
- [UDP](transports/udp.md) — UDP transport
- [WebRTC](transports/webrtc.md) — WebRTC DataChannel
- [Serial](transports/serial.md) — Serial/UART transport
- [BLE](transports/ble.md) — Bluetooth Low Energy

## Configuration Reference

- [Router Configuration](configuration/router-config.md) — Router options
- [Bridge Configuration](configuration/bridge-config.md) — Bridge options
- [Feature Flags](configuration/feature-flags.md) — Cargo feature flags
