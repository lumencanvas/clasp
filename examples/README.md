# CLASP Examples

This directory contains example code demonstrating how to use CLASP in various scenarios.

## JavaScript Examples

### simple-publisher.js

Demonstrates publishing values and events to a CLASP server.

```bash
cd examples/js
npm install @clasp-to/core
node simple-publisher.js
```

Features demonstrated:
- Setting parameter values
- Emitting events
- Streaming high-rate data
- Atomic bundles
- Scheduled bundles

### simple-subscriber.js

Demonstrates subscribing to values and events from a CLASP server.

```bash
cd examples/js
npm install @clasp-to/core
node simple-subscriber.js
```

Features demonstrated:
- Subscribing to specific addresses
- Wildcard subscriptions (`*` and `**`)
- Rate-limited subscriptions
- Change threshold filtering (epsilon)
- Getting values (async)
- Checking cached values (sync)
- Unsubscribing

## Rust Examples

### basic-client.rs

Comprehensive Rust client example.

```bash
cargo run --example basic-client
```

Or add to your project:

```toml
[dependencies]
clasp-client = "0.1"
```

Features demonstrated:
- Builder pattern for client creation
- Setting parameters
- Subscribing with callbacks
- Emitting events
- Streaming data
- Getting values
- Atomic and scheduled bundles

## Docker Compose

### docker-compose.yml

Complete development environment with CLASP Router and MQTT broker.

```bash
# Start basic setup
docker-compose up -d clasp-router mqtt

# Start with Redis for distributed state
docker-compose --profile distributed up -d

# Stop
docker-compose down
```

Services:
- **clasp-router**: Core CLASP message router (port 7330)
- **mqtt**: Mosquitto MQTT broker (port 1883)
- **redis**: Redis for distributed state (port 6379, optional)

## Environment Variables

All examples support the following environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `CLASP_URL` | `ws://localhost:7330` | CLASP server WebSocket URL |

## Running a CLASP Server

To run these examples, you need a CLASP server. Options:

1. **Desktop App**: Download from [releases](https://github.com/lumencanvas/clasp/releases)

2. **Docker**:
   ```bash
   docker run -p 7330:7330 lumencanvas/clasp-router
   ```

3. **From Source**:
   ```bash
   cargo run -p clasp-router-server
   ```

## More Examples

For more complex integration examples, see:
- [TouchOSC Integration](../docs/integrations/touchosc.md)
- [Resolume Integration](../docs/integrations/resolume.md)
- [QLab Integration](../docs/integrations/qlab.md)
