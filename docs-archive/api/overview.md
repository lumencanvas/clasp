---
title: "Overview"
description: "Documentation page for the CLASP protocol."
section: api
order: 1
---
## CLASP API Overview

CLASP is a transport-agnostic protocol for low-latency, stateful control of creative systems (lighting, video, audio, sensors, and more). The API surface is intentionally small and consistent across languages so that the same mental model applies whether you are writing Rust, JavaScript, Python, or embedded C.

- **Signals, not raw messages**: Everything is expressed as Signals (Param, Event, Stream, Gesture, Timeline) rather than opaque packets.
- **Stateful by default**: Parameters have authoritative values with revisions; late joiners can query current state.
- **Transport-agnostic**: The same API works over WebSocket, WebRTC DataChannel, UDP, QUIC, BLE, Serial, and custom transports.
- **Discovery-aware**: Routers and bridges support discovery so clients can find peers automatically where possible.
- **Security built in**: Capability tokens and encryption are part of the core model, not bolted on.

Most client APIs follow the same flow:

1. **Connect** to a CLASP router (or peer) over a chosen transport.
2. **Announce** or discover available signals.
3. **READ** state (GET/SNAPSHOT) and **SUBSCRIBE** to updates.
4. **WRITE** state and emit events/streams/gestures.
5. Optionally use **BUNDLES**, **P2P**, and advanced timing/security features.

For full protocol details, see `CLASP-Protocol.md`. The files in `docs/api/common/` map those concepts directly to APIs in each language.

