---
title: Reference
description: Complete reference documentation for CLASP
order: 1
---

# Reference

Complete reference documentation for CLASP CLIs, protocol specification, configuration schemas, and SDK APIs.

## CLIs

- [Relay CLI Reference](relay-cli.md) -- complete `clasp-relay` command-line reference
- [CLASP CLI Reference](clasp-cli.md) -- complete `clasp` command-line reference

## Protocol

- [Wire Protocol Specification](protocol-spec.md) -- binary frame format, message types, value encoding

## Configuration Schemas

- [App Config Schema](app-config-schema.md) -- JSON schema for scopes, write rules, snapshot transforms, and rate limits
- [Rules Schema](rules-schema.md) -- complete JSON schema for the rules engine
- [Router Config](router-config.md) -- `RouterConfig` reference for embedding routers in Rust
## Transports

- [Transport Guide](../core/transports.md) -- WebSocket, QUIC, TCP, UDP, Serial, BLE configuration and usage

## SDK APIs

### Rust

- [Rust Crates Overview](rust-crates.md) -- all CLASP Rust crates, dependencies, and feature flags

### JavaScript

- [JavaScript API](js-api.md) -- complete `@clasp-to/core` API reference

### Python

- [Python API](python-api.md) -- complete `clasp-to` API reference

## Bridges

- [OSC Bridge](../protocols/osc.md) -- OSC to CLASP mapping
- [MIDI Bridge](../protocols/midi.md) -- MIDI to CLASP mapping
- [Art-Net Bridge](../protocols/artnet.md) -- Art-Net to CLASP mapping
- [DMX Bridge](../protocols/dmx.md) -- DMX to CLASP mapping
- [MQTT Bridge](../protocols/mqtt.md) -- MQTT to CLASP mapping
- [sACN Bridge](../protocols/sacn.md) -- sACN to CLASP mapping
- [HTTP Bridge](../protocols/http.md) -- HTTP to CLASP mapping

## Next Steps

Start with whichever reference you need. For a guided introduction, see the [Getting Started](../getting-started/README.md) guide. For architecture context, see the [Architecture](../concepts/architecture.md) explanation.
