---
title: Relay Server
description: The CLASP relay -- production server with auth, persistence, and federation
order: 3
---

# Relay Server

The relay server (`clasp-relay`) is CLASP's production deployment target. It wraps a CLASP router with authentication, persistence, federation, a rules engine, and embedded protocol servers. It is a standalone Rust binary located in `deploy/relay/`, separate from the main workspace.

## Architecture

```
Clients (WS/QUIC) ──┐
MQTT Clients ────────┤
OSC Clients ─────────┼── clasp-relay ── Router ── State Store
HTTP Auth ───────────┤                          ── Journal
Federation Peers ────┘                          ── Rules Engine
```

The relay accepts connections over WebSocket, QUIC, MQTT, and OSC. All protocol traffic is normalized into CLASP signals and routed through the same state store. Auth is handled by a separate HTTP server on its own port.

## Quick Start

With Docker:

```bash
docker run -p 7330:7330 clasp-relay
```

From source:

```bash
cd deploy/relay
cargo run --release --features full
```

Both start a relay with default settings: WebSocket on port 7330, no auth, no persistence.

## Core Configuration

| Flag                | Default       | Description                                |
| ------------------- | ------------- | ------------------------------------------ |
| `--ws-port`         | `7330`        | WebSocket listener port                    |
| `--host`            | `0.0.0.0`    | Bind address                               |
| `--name`            | (hostname)    | Human-readable relay name                  |
| `--max-sessions`    | `1000`        | Maximum concurrent client sessions         |
| `--session-timeout` | `300`         | Idle session timeout in seconds            |
| `--verbose`         |               | Enable verbose logging                     |
| `--no-websocket`    |               | Disable the WebSocket listener             |

## Multi-Protocol

The relay can serve MQTT, OSC, and QUIC clients alongside WebSocket. Each protocol server maps its native addressing into the CLASP namespace.

```bash
clasp-relay \
  --mqtt-port 1883 \
  --osc-port 8000 \
  --quic-port 7331 --cert cert.pem --key key.pem
```

MQTT clients publishing to `sensors/temp` appear as `/mqtt/sensors/temp` for WebSocket subscribers. OSC messages sent to `/stage/dimmer` appear as `/osc/stage/dimmer`. The namespace prefix is configurable:

```bash
clasp-relay \
  --mqtt-port 1883 --mqtt-namespace /devices \
  --osc-port 8000 --osc-namespace /stage
```

QUIC requires TLS certificates via `--cert` and `--key`. For detailed transport configuration (buffer sizes, keep-alive, certificate verification modes), see [Transports](../core/transports.md).

## TTL Configuration

Signals and parameters have a default time-to-live of 1 hour (3600 seconds). After the TTL expires, values are eligible for cleanup.

| Flag           | Default | Description                       |
| -------------- | ------- | --------------------------------- |
| `--param-ttl`  | `3600`  | TTL for parameter values (seconds)|
| `--signal-ttl` | `3600`  | TTL for signal values (seconds)   |
| `--no-ttl`     |         | Disable TTL expiration entirely   |

## Feature Flags

Optional subsystems are compiled in via Cargo feature flags. Each flag unlocks additional CLI options.

| Feature      | What it enables                         | Required CLI options                      |
| ------------ | --------------------------------------- | ----------------------------------------- |
| `journal`    | SQLite event journal                    | `--journal <path>` or `--journal-memory`  |
| `caps`       | Capability token delegation             | `--trust-anchor <path>` (repeatable)      |
| `registry`   | Persistent entity registry              | `--registry-db <path>`                    |
| `rules`      | JSON signal filtering rules             | `--rules <path>`                          |
| `federation` | Inter-relay federation                  | `--federation-hub`, `--federation-id`     |
| `full`       | All of the above                        | (any of the above)                        |

Build with specific features:

```bash
cargo build --release --features journal,rules
```

## Common Configurations

**Basic (no auth):**

```bash
clasp-relay
```

**With auth:**

```bash
clasp-relay \
  --auth-port 7350 \
  --cors-origin https://app.example.com \
  --admin-token ./secrets/admin.token
```

**With auth and journal:**

```bash
clasp-relay \
  --auth-port 7350 \
  --cors-origin https://app.example.com \
  --admin-token ./secrets/admin.token \
  --journal ./data/journal.db \
  --persist ./data/state.db
```

**Full production setup:**

```bash
clasp-relay \
  --auth-port 7350 \
  --cors-origin https://app.example.com \
  --admin-token ./secrets/admin.token \
  --journal ./data/journal.db \
  --persist ./data/state.db --persist-interval 30 \
  --app-config ./config/app.json \
  --mqtt-port 1883 \
  --param-ttl 3600 \
  --max-sessions 500 \
  --metrics-port 9090
```

## Rendezvous

The relay includes a built-in rendezvous server for client discovery.

| Flag                | Default | Description                          |
| ------------------- | ------- | ------------------------------------ |
| `--rendezvous-port` | `7340`  | UDP rendezvous listener port         |
| `--rendezvous-ttl`  | `300`   | Registration TTL in seconds          |

Rendezvous is enabled by default. Clients can discover the relay via UDP broadcast on the configured port.

## Graceful Shutdown

The relay supports graceful shutdown with a configurable drain timeout:

```bash
clasp-relay --drain-timeout 30
```

On SIGTERM or SIGINT, the relay stops accepting new connections and waits up to `--drain-timeout` seconds for existing sessions to close before exiting.

## Next Steps

- [Docker Deployment](docker.md) -- containerize the relay
- [Auth setup](../auth/README.md) -- configure authentication
- [Production Checklist](production-checklist.md) -- verify your deployment
