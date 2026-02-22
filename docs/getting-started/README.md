---
title: Installation
description: Install CLASP for JavaScript, Python, Rust, or Docker
order: 1
---

# Installation

CLASP provides client libraries for JavaScript, Python, and Rust, plus a standalone relay server for production deployment. Install the client library for your language and optionally run a local router for development.

## Client Libraries

### JavaScript

```bash
npm install @clasp-to/core
```

Current version: **3.4.0**. Requires Node.js 18+ or any modern browser.

Verify the installation:

```javascript
import { ClaspBuilder } from '@clasp-to/core';

const client = await new ClaspBuilder('ws://localhost:7330').withName('test').connect();
client.set('/hello', 'world');
```

### Python

```bash
pip install clasp-to
```

Current version: **3.3.1**. Requires Python 3.9+.

Verify the installation:

```python
from clasp import Clasp

client = Clasp('ws://localhost:7330', name='test')
await client.connect()
client.set('/hello', 'world')
```

### Rust

```bash
cargo add clasp-client
```

Verify the installation:

```rust
use clasp_client::ClaspClient;

let client = ClaspClient::connect("ws://localhost:7330", "test").await?;
client.set("/hello", "world").await?;
```

## Router / Server

Client libraries need a running router to connect to. Pick one of the following methods.

### For Development

Install the CLI and start a local router:

```bash
cargo install clasp-cli
clasp server
```

The router starts on `ws://localhost:7330` by default.

### For Production

Pull and run the official Docker image:

```bash
docker pull ghcr.io/lumencanvas/clasp-relay
docker run -p 7330:7330 ghcr.io/lumencanvas/clasp-relay
```

### Build from Source

Clone the repository and build the relay binary:

```bash
cd deploy/relay
cargo build --release
./target/release/clasp-relay
```

## Docker (Quickest Start)

If you just want something running immediately:

```bash
docker run -p 7330:7330 ghcr.io/lumencanvas/clasp-relay
```

This starts a router with default configuration on port 7330. No auth, no persistence -- suitable for local development only. See [Relay Server](../deployment/relay.md) for production configuration.

## Example Projects

The `examples/` directory contains working code across all three SDKs. Each example is self-contained and can be run directly.

### Basics

| Example | JS | Python | Rust |
|---------|:--:|:------:|:----:|
| Publish values | `simple-publisher.js` | -- | `basic-client.rs` |
| Subscribe to values | `simple-subscriber.js` | -- | -- |
| All signal types | `signal-types.js` | `signal_types.py` | `signal_types.rs` |

### State & Coordination

| Example | JS | Python | Rust |
|---------|:--:|:------:|:----:|
| Bundles & scheduling | `bundles-and-scheduling.js` | `bundles_and_scheduling.py` | `bundles_and_scheduling.rs` |
| Late-joiner sync | `late-joiner.js` | `late_joiner.py` | `late_joiner.rs` |
| Locks | `locks.js` | -- | -- |
| Discovery | `discovery.js` | -- | -- |

### P2P & Video

| Example | JS | Python | Rust |
|---------|:--:|:------:|:----:|
| P2P WebRTC | `p2p-webrtc.js` | `p2p_webrtc.py` | `p2p_webrtc.rs` |
| Video (P2P) | `video-p2p.html` | -- | -- |
| Video (relay) | `video-relay.html` | -- | -- |

### Input & Security

| Example | JS | Python | Rust |
|---------|:--:|:------:|:----:|
| Gestures (touch/pen) | `gestures.js` | -- | -- |
| Security tokens | `security-tokens.js` | `security_tokens.py` | `security_tokens.rs` |

### Embedded

| Example | JS | Python | Rust |
|---------|:--:|:------:|:----:|
| Embedded server | `embedded-server.js` | `embedded_server.py` | `embedded-server.rs` |

All examples are in the [`examples/`](https://github.com/lumencanvas/clasp/tree/main/examples) directory.

## Next Steps

- [First Connection](first-connection.md) -- send your first signal through CLASP in 5 minutes
