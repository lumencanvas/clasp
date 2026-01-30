# CLASP Examples

Comprehensive examples demonstrating how to use CLASP in various scenarios across JavaScript, Python, and Rust.

## Quick Start

```bash
# Start a CLASP router first
cargo run -p clasp-router-server -- --listen 0.0.0.0:7330

# Then run any example
```

## JavaScript Examples

Located in `examples/js/`. Run with Node.js 18+.

```bash
cd examples/js
npm install @clasp-to/core
```

| Example | Description |
|---------|-------------|
| `simple-publisher.js` | Basic publishing: set params, emit events, stream data |
| `simple-subscriber.js` | Subscribing with wildcards, rate limiting, epsilon filtering |
| `signal-types.js` | All 5 signal types: Param, Event, Stream, Gesture, Timeline |
| `bundles-and-scheduling.js` | Atomic bundles, scheduled execution, cancellation |
| `gestures.js` | Touch/pen input, multi-touch, pressure sensitivity, coalescing |
| `discovery.js` | mDNS discovery, UDP broadcast, manual connection |
| `locks.js` | Exclusive control, lock contention, hierarchical locks |
| `late-joiner.js` | State synchronization for late-joining clients |
| `security-tokens.js` | CPSK tokens, scoped permissions, authentication |
| `p2p-webrtc.js` | Peer-to-peer via WebRTC DataChannels |
| `embedded-server.js` | Integrating CLASP into your Node.js app |
| `video-relay.html` | Video streaming via CLASP relay (WebCodecs H.264) |
| `video-p2p.html` | Video calling via WebRTC with CLASP signaling |

### Run an Example

```bash
node signal-types.js
node bundles-and-scheduling.js
# etc.
```

### Video Examples (Browser)

The video examples are standalone HTML files. Serve them over localhost (camera requires secure context):

```bash
# Using Python
python3 -m http.server 8000 --directory examples/js

# Or any static file server, then open:
# http://localhost:8000/video-relay.html
# http://localhost:8000/video-p2p.html
```

**video-relay.html** - Demonstrates CLASP relay mode video streaming:
camera capture, WebCodecs H.264 encoding, frame chunking, CLASP stream
transport, chunk reassembly, WebCodecs decoding, and canvas rendering.
Requires a browser with WebCodecs support (Chrome 94+).

**video-p2p.html** - Demonstrates P2P video calling using WebRTC with
CLASP for signaling (offer/answer/ICE exchange). Room-based presence
via CLASP params. Works in any modern browser with WebRTC support.

## Python Examples

Located in `examples/python/`. Requires Python 3.9+.

```bash
pip install clasp-to
```

| Example | Description |
|---------|-------------|
| `signal_types.py` | All 5 signal types with async/await |
| `bundles_and_scheduling.py` | Atomic and scheduled bundles |
| `late_joiner.py` | State sync demonstration |
| `security_tokens.py` | Token-based authentication |
| `p2p_webrtc.py` | Peer-to-peer connections |
| `embedded_server.py` | Integrating CLASP into Python apps |

### Run an Example

```bash
python signal_types.py
python bundles_and_scheduling.py
# etc.
```

## Rust Examples

Located in `examples/rust/`. These are cargo examples.

```bash
# Add to your Cargo.toml
[dependencies]
clasp-client = "3.0"
clasp-router = "3.0"  # If embedding a router
```

| Example | Description |
|---------|-------------|
| `basic-client.rs` | Comprehensive client example |
| `signal_types.rs` | All signal types in Rust |
| `bundles_and_scheduling.rs` | Atomic and scheduled bundles |
| `late_joiner.rs` | State synchronization |
| `security_tokens.rs` | Token authentication |
| `p2p_webrtc.rs` | WebRTC peer-to-peer |
| `embedded-server.rs` | Embedding router in your app |

### Run an Example

```bash
cargo run --example basic-client
cargo run --example signal_types
cargo run --example bundles_and_scheduling
# etc.
```

## Feature Coverage

| Feature | JS | Python | Rust |
|---------|:--:|:------:|:----:|
| Params (stateful values) | signal-types | signal_types | signal_types |
| Events (one-shot triggers) | signal-types | signal_types | signal_types |
| Streams (high-frequency) | signal-types | signal_types | signal_types |
| Gestures (phased input) | gestures | signal_types | signal_types |
| Timelines (automation) | signal-types | signal_types | signal_types |
| Atomic Bundles | bundles-and-scheduling | bundles_and_scheduling | bundles_and_scheduling |
| Scheduled Bundles | bundles-and-scheduling | bundles_and_scheduling | bundles_and_scheduling |
| Wildcard Subscriptions | simple-subscriber | signal_types | basic-client |
| Late Joiner Sync | late-joiner | late_joiner | late_joiner |
| Locks | locks | - | - |
| Discovery (mDNS/UDP) | discovery | - | - |
| Security Tokens | security-tokens | security_tokens | security_tokens |
| P2P WebRTC | p2p-webrtc | p2p_webrtc | p2p_webrtc |
| Embedded Server | embedded-server | embedded_server | embedded-server |
| Video Relay (WebCodecs) | video-relay.html | - | - |
| Video P2P (WebRTC) | video-p2p.html | - | - |

## Docker Compose

A complete development environment is available:

```bash
# Start CLASP router + MQTT broker
docker-compose up -d clasp-router mqtt

# Start with Redis for distributed state
docker-compose --profile distributed up -d

# Stop
docker-compose down
```

## Environment Variables

All examples support:

| Variable | Default | Description |
|----------|---------|-------------|
| `CLASP_URL` | `ws://localhost:7330` | CLASP server URL |
| `CLASP_TOKEN` | (none) | Authentication token |

## Running a CLASP Server

Options:

1. **Desktop App**: Download from [releases](https://github.com/lumencanvas/clasp/releases)

2. **Docker**:
   ```bash
   docker run -p 7330:7330 lumencanvas/clasp-router
   ```

3. **From Source**:
   ```bash
   cargo run -p clasp-router-server
   ```

## More Resources

- [Tutorials](../docs/tutorials/) - Step-by-step learning guides
- [How-To Guides](../docs/how-to/) - Solve specific problems
- [API Reference](../docs/reference/api/) - Complete API documentation
- [Integrations](../docs/integrations/) - TouchOSC, Resolume, QLab, etc.
