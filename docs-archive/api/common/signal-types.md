---
title: "Signal Types"
description: "Documentation page for the CLASP protocol."
section: api
order: 5
---
## CLASP Signal Types

CLASP models control traffic as **Signals**, not just messages. Each signal type has clear semantics and quality-of-service expectations.

- **Param**: Stateful values (e.g. `/light/1/brightness`, `/layer/0/opacity`).
  - Backed by authoritative state with revisions.
  - Late joiners can query the current value via `GET`/`SNAPSHOT`.
  - Typically sent with **Confirm (Q1)** QoS.

- **Event**: Ephemeral triggers (e.g. `/cue/fire`, `/button/1/press`).
  - No persistent state; consumers that are offline simply miss the event.
  - Typically sent with **Confirm (Q1)** QoS so producers know it was delivered.

- **Stream**: High‑rate time series (e.g. sensors, faders at 60–240 Hz).
  - Designed for lossy, low‑latency delivery.
  - Typically mapped to **Fire (Q0)** QoS; receivers tolerate drops.
  - Supports options like `maxRate`, `epsilon`, and batching.

- **Gesture**: Phased input with semantic stages (e.g. touch begin/move/end).
  - Encodes phases to make multi-touch and drawing tools easier to implement.
  - Usually transported like Streams, but with explicit phase markers.

- **Timeline**: Automation and scheduled changes over time.
  - Represents sequences of future state changes.
  - Often used together with Bundles and precise timestamps.

Language‑specific API docs show how each signal type is represented in code (types, helpers, and idioms) for that runtime.

