---
title: Local Development
description: Run CLASP locally for development
order: 2
---

# Local Development

For local development, use `clasp server` for a quick zero-config router or `docker compose` for a full relay setup with auth and persistence.

## Quick Router

`clasp server` starts a WebSocket router on port 7330. No auth, no persistence, no feature flags. It is the fastest way to test SDK code against a live router.

```bash
cargo install clasp-cli
clasp server
# Router listening on ws://localhost:7330
```

The router accepts all connections, routes all signals, and stores state in memory. When you stop it, everything is gone.

## Docker Compose

For a local environment closer to production, use Docker Compose to run the relay with auth, persistence, and any features you need.

```yaml
services:
  relay:
    image: clasp-relay
    build:
      context: deploy/relay
      args:
        FEATURES: "full"
    ports:
      - "7330:7330"
      - "7350:7350"
    command: >
      clasp-relay
        --auth-port 7350
        --persist /data/state.db
        --cors-origin http://localhost:5173
    volumes:
      - relay-data:/data

volumes:
  relay-data:
```

Start it:

```bash
docker compose up
```

This gives you a relay with auth on port 7350, state snapshots in a persistent volume, and CORS configured for a Vite dev server on port 5173.

## Environment Variables

| Variable     | Default | Description                              |
| ------------ | ------- | ---------------------------------------- |
| `RUST_LOG`   | `info`  | Log level: `trace`, `debug`, `info`, `warn`, `error` |
| `LOG_FORMAT` | `text`  | Set to `json` for structured log output  |

For verbose development logging:

```bash
RUST_LOG=debug clasp server
```

## Connecting

Once the router or relay is running, connect from any SDK.

JavaScript:

```javascript
import { ClaspBuilder } from '@clasp-to/core';

const clasp = new ClaspBuilder('ws://localhost:7330').build();
await clasp.connect();
```

Python:

```python
from clasp import Clasp

clasp = Clasp('ws://localhost:7330')
await clasp.connect()
```

Rust:

```rust
use clasp_client::ClaspClient;

let client = ClaspClient::connect("ws://localhost:7330").await?;
```

## Next Steps

- [Relay Server](relay.md) -- understand the relay's full configuration
- [Docker Deployment](docker.md) -- build and run relay containers for staging or production
