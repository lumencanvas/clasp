# Handoff - 2026-02-21: Undocumented Features Added to Docs

## Overview

Added documentation for ~10 features that were implemented in the codebase but missing from the 49-page docs site. Two new pages created, six existing pages edited. Docs build passes with 51 pages validated, 0 issues.

**Result: 2 new pages, 6 edited pages, 1 bug fix (YAML -> JSON in architecture.md). Build: 51 docs, 0 broken links.**

## New Pages

### `docs/core/p2p.md` — P2P & WebRTC

Full documentation of direct peer-to-peer communication:
- When to use P2P vs relay (comparison table)
- `P2PManager` API with examples in all three SDKs (JS, Python, Rust)
- Constructor options: `peerId`, `rendezvousUrl`, `iceServers`, `useUnreliableChannel`
- Registration with rendezvous server (tags + metadata)
- Peer discovery with tag filtering
- Connection establishment (`connect(peerId)`)
- Peer operations: `set()`, `stream()`, `emit()`, `on()`
- Dual channels (reliable for state, unreliable for streams)
- ICE/STUN/TURN configuration including private TURN servers
- Auto-fallback to relay on P2P failure
- Complete working example adapted from `examples/js/p2p-webrtc.js`
- Video streaming section covering both P2P video (WebRTC PeerConnection) and relay video (WebCodecs + Stream signals)

**Sources**: `examples/js/p2p-webrtc.js`, `examples/python/p2p_webrtc.py`, `examples/rust/p2p_webrtc.rs`, `examples/js/video-p2p.html`, `examples/js/video-relay.html`

### `docs/core/timing.md` — Clock Sync & Timing

Full documentation of timing primitives:
- `ClockSync`: NTP 4-timestamp algorithm, EMA smoothing (alpha=0.3), quality scoring
  - `process_sync(t1, t2, t3, t4)`, `offset()`, `rtt()`, `jitter()`, `quality()`, `needs_sync()`, `server_time()`, `to_server_time()`, `to_local_time()`
- Quality score breakdown (RTT 40%, jitter 40%, samples 20%)
- JS SDK auto-sync via HELLO/WELCOME + SYNC messages
- `SessionTime`: `elapsed()`, `start_time()`, `to_unix()`, `from_unix()`
- `JitterBuffer<T>`: `new(capacity, window_ms)`, `push()`, `pop()`, `drain_ready()`, `len()`, `depth_us()`
- Worked example: synchronized lighting cue across two venues using scheduled bundles

**Source**: `crates/clasp-core/src/time.rs`

## Edited Pages

### `docs/reference/protocol-spec.md` — Error Codes Section

Appended full error codes reference before the "Backward Compatibility" section:
- 5 ranges: Protocol (100s), Address (200s), Auth (300s), State (400s), Server (500s)
- 15 codes total with numeric value, name, and description
- Matches `crates/clasp-core/src/error.rs` `ErrorCode` enum exactly
- JS SDK error handling example
- Recommended handling strategy per category

### `docs/getting-started/README.md` — Example Projects Section

Added categorized tables of all 28 example files across JS/Python/Rust:
- Basics (publisher, subscriber, signal types)
- State & Coordination (bundles, late-joiner, locks, discovery)
- P2P & Video (p2p-webrtc, video-p2p, video-relay)
- Input & Security (gestures, security tokens)
- Embedded (embedded server)

### `docs/sdk/javascript.md` — Reconnection + Examples

Added two new sections:
- **Reconnection & Connection Lifecycle**: connection state diagram, auto-reconnect behavior (exponential backoff 1.5x, 30s cap, 10 max attempts), subscription re-establishment on reconnect, manual close
- **Examples**: table of all 13 JS example files with descriptions
- Updated Next Steps to include P2P link

**Source**: `bindings/js/packages/clasp-core/src/client.ts` (scheduleReconnect, resubscribeAll)

### `docs/sdk/python.md` — Reconnection + Examples

Added two new sections:
- **Reconnection & Connection Lifecycle**: auto-reconnect with decorator-style event handlers
- **Examples**: table of all 6 Python example files
- Updated Next Steps to include P2P link

### `docs/sdk/rust.md` — Reconnection + Examples

Added two new sections:
- **Reconnection & Connection Lifecycle**: builder-based reconnect config, event callbacks
- **Examples**: table of all 7 Rust example files with `cargo run --example` instructions
- Updated Next Steps to include P2P link

### `docs/core/signals.md` — Gesture Deep Dive

Expanded the Gesture section with:
- Phased model explanation (physical input mapping)
- Pressure and metadata example (Wacom pen with tiltX/tiltY, custom metadata)
- Multi-touch example (pinch-zoom with two simultaneous gesture IDs)
- Coalescing behavior (240Hz input -> ~24 delivered updates, 90% bandwidth reduction)
- Receiving gestures with `onGesture` subscriber example

**Source**: `examples/js/gestures.js`

## Bug Fix

### `docs/concepts/architecture.md` — YAML -> JSON

The architecture page incorrectly stated app config reads from "TOML/YAML". The actual format is JSON (`--app-config <path>`), as confirmed by `deploy/relay/src/app_config.rs:3`. Fixed to read: "reads declarative write rules and visibility from JSON (`--app-config`)"

## Sidebar

No sidebar changes needed. `DocsSidebar.vue` auto-discovers pages from the Vite docs manifest plugin. The two new `docs/core/` pages appear automatically under CORE CONCEPTS, sorted by their `order` frontmatter (p2p: 6, timing: 7).

## Build Verification

```
cd apps/docs && npx vite build
[clasp-docs] 51 docs validated, 0 issues
✓ built in 667ms
```

## What Was NOT Changed

- No code changes to any Rust crate or JS package
- No new example files (only linked existing ones)
- No changes to the Vite plugin or build system
- No changes to CSS/styling
