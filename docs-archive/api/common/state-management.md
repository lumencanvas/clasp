---
title: "State Management"
description: "Documentation page for the CLASP protocol."
section: api
order: 6
---
## State Management in CLASP

CLASP treats **state as a first‑class concept**. Parameters have authoritative values with revision numbers, and clients interact with that state through a small set of verbs.

### Core Operations

- **SET** – Change the value of a Param at a given address.
- **GET** – Retrieve the current value at an address.
- **SNAPSHOT** – Retrieve a consistent snapshot of many addresses at once.
- **SUBSCRIBE** – Receive updates as state changes over time.

Every stateful change carries a **revision** so that:

- Clients can detect out‑of‑date writes.
- Conflict resolution strategies (last‑write‑wins, max/min, locks, custom) have the information they need.

### Conflict Resolution and Locks

Multi‑controller systems frequently need to coordinate writes:

- **Last‑write‑wins (LWW)** for simple parameters.
- **Max/Min** strategies for things like meters.
- **Explicit locks** when a controller needs exclusive control.
- **Application‑defined merge** for complex structures.

The core protocol defines the primitives; application code and language bindings provide higher‑level helpers that map cleanly onto these rules.

