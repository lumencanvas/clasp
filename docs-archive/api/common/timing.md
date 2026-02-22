---
title: "Timing"
description: "Documentation page for the CLASP protocol."
section: api
order: 7
---
## Timing and Scheduling

Precise timing is one of CLASP’s core strengths: it is designed for live shows, installations, and other time‑sensitive systems.

### Clock Model

- Each session maintains a **monotonic clock** (microseconds since session start).
- Routers and clients run a **clock sync** protocol so scheduled messages execute at the intended wall‑clock time.

### Scheduled Bundles

Any Bundle can carry a timestamp:

- The router (or peer) applies all messages in the bundle atomically at (or as close as possible to) the requested time.
- Clients can schedule timelines, cues, and transitions without micro‑managing individual messages.

### Jitter Handling

On unreliable networks (WiFi, WAN), end‑to‑end jitter is inevitable:

- Receivers can use **jitter buffers** for Streams.
- Scheduled Bundles and Timelines rely on clock sync rather than “send at exactly the right millisecond”.

Language‑specific docs include helpers for clock sync, scheduling Bundles, and configuring jitter buffers where relevant.

