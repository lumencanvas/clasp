---
title: "Transports"
description: "Documentation page for the CLASP protocol."
section: api
order: 8
---
## Transports

CLASP is **transport‑agnostic**: the same frame format can be carried over any byte stream or datagram transport.

Common transports include:

- **WebSocket** – Universal baseline, especially for browsers.
- **WebRTC DataChannel** – Low‑latency P2P with configurable reliability.
- **QUIC** – Multiplexed, connection‑migrating transport for mobile and WAN.
- **UDP** – Minimal overhead for LAN and embedded devices.
- **BLE** – Low‑power wireless for controllers and wearables.
- **Serial** – Direct hardware integration for devices like DMX interfaces.

The protocol defines **QoS levels**, frame format, and encoding; individual transports map those onto their own semantics (e.g. ordered/reliable vs unordered/unreliable channels). Language‑specific APIs document which transports are supported and how to configure them.

