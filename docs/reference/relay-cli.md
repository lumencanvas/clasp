---
title: Relay CLI Reference
description: Complete clasp-relay command-line reference
order: 2
---

# Relay CLI Reference

Complete reference for the `clasp-relay` binary. All flags, defaults, and environment variables.

The relay is a standalone binary (not part of the Cargo workspace) located in `deploy/relay/`. It can also be used as a library crate (`clasp-relay`) for embedding into other Rust applications -- see `RelayConfig` in the source for the programmatic API.

## Usage

```
clasp-relay [OPTIONS]
```

## Core

| Flag | Default | Description |
|------|---------|-------------|
| `-p`, `--ws-port` (alias: `--port`) | `7330` | WebSocket listen port |
| `--host` | `0.0.0.0` | Listen host address |
| `-n`, `--name` | `CLASP Relay` | Server name (shown in WELCOME message) |
| `-v`, `--verbose` | off | Enable verbose logging |
| `--max-sessions` | `1000` | Maximum concurrent clients (0 = unlimited) |
| `--session-timeout` | `300` | Session timeout in seconds |
| `--no-websocket` | off | Disable WebSocket listener (use other protocols only) |

## Protocols

| Flag | Default | Description |
|------|---------|-------------|
| `--quic-port` | none | QUIC listen port (enables QUIC transport; requires `--cert` and `--key`) |
| `--mqtt-port` | none | MQTT listen port (enables MQTT server adapter) |
| `--mqtt-namespace` | `/mqtt` | MQTT namespace prefix for CLASP address mapping |
| `--osc-port` | none | OSC listen port (enables OSC server adapter) |
| `--osc-namespace` | `/osc` | OSC namespace prefix for CLASP address mapping |
| `--cert` | none | TLS certificate file (PEM format, for QUIC and MQTTS) |
| `--key` | none | TLS private key file (PEM format, for QUIC and MQTTS) |

## TTL

| Flag | Default | Description |
|------|---------|-------------|
| `--param-ttl` | `3600` | Parameter TTL in seconds (0 = disabled). Parameters not updated within this time are automatically removed. |
| `--signal-ttl` | `3600` | Signal TTL in seconds (0 = disabled). Signal definitions not accessed within this time are automatically removed. |
| `--no-ttl` | off | Disable all TTL expiration (parameters and signals persist indefinitely) |

## Auth

| Flag | Default | Description |
|------|---------|-------------|
| `--auth-port` | none | Auth HTTP server port (enables authentication subsystem) |
| `--auth-db` | `relay-auth.db` | Auth database path (SQLite) |
| `--cors-origin` | none | Allowed CORS origin(s) for the auth API (comma-separated). If not set, CORS is permissive (development only). |
| `--admin-token` | none | Admin token file path. If the file exists, reads the token from it. If not, generates a new admin token and writes it to the file. The token is registered with `admin:/**` scope (no expiry). |
| `--token-ttl` | `86400` | Default TTL for CPSK tokens in seconds (0 = no default expiry). Tokens registered without an explicit expiry use this duration. |

## Persistence

| Flag | Default | Description |
|------|---------|-------------|
| `--persist` | none | Path to state snapshot file (enables persistence across restarts) |
| `--persist-interval` | `30` | Snapshot interval in seconds |

## Rendezvous

| Flag | Default | Description |
|------|---------|-------------|
| `--rendezvous-port` | `7340` | Rendezvous server port for WAN discovery (serves `/api/v1/*`). Set to 0 to disable. |
| `--rendezvous-ttl` | `300` | Rendezvous TTL in seconds (how long device registrations last) |

## Journal

Requires: `--features journal`

| Flag | Default | Description |
|------|---------|-------------|
| `--journal` | none | SQLite journal path for state persistence and replay |
| `--journal-memory` | off | Use in-memory journal (ring buffer, no on-disk persistence) |

## Capabilities

Requires: `--features caps`

| Flag | Default | Description |
|------|---------|-------------|
| `--trust-anchor` | none | Trust anchor public key file(s) for capability tokens (32-byte Ed25519). Repeatable -- specify multiple times for multiple anchors. |
| `--cap-max-depth` | `5` | Maximum delegation chain depth for capability tokens |

## Registry

Requires: `--features registry`

| Flag | Default | Description |
|------|---------|-------------|
| `--registry-db` | none | SQLite database path for the entity registry |

## Rules

Requires: `--features rules`

| Flag | Default | Description |
|------|---------|-------------|
| `--rules` | none | JSON file containing rule definitions |

## App Config

| Flag | Default | Description |
|------|---------|-------------|
| `--app-config` | auto-detect | JSON file defining scopes, write rules, and snapshot rules. If not specified, auto-detects from `/etc/clasp/` or `./config/` (single JSON file in the directory). See [App Config Schema](app-config-schema.md). |

## Federation

Requires: `--features federation`

| Flag | Default | Description |
|------|---------|-------------|
| `--federation-hub` | none | Hub WebSocket URL for federation leaf mode (e.g., `ws://hub:7330`) |
| `--federation-id` | none | Local router identity for federation |
| `--federation-namespace` | none | Namespace pattern(s) owned by this router. Repeatable -- specify multiple times for multiple namespaces. |
| `--federation-token` | none | Auth token to present to the federation hub |

## Metrics

| Flag | Default | Description |
|------|---------|-------------|
| `--metrics-port` | none | Prometheus metrics HTTP port (enables `/metrics` endpoint) |

## Health

| Flag | Default | Description |
|------|---------|-------------|
| `--health-port` | none | Health check HTTP port (enables `/healthz` and `/readyz` endpoints) |

## Shutdown

| Flag | Default | Description |
|------|---------|-------------|
| `--drain-timeout` | `30` | Graceful shutdown drain timeout in seconds. After receiving SIGTERM, the server waits this long before force-closing connections. |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `RUST_LOG` | Log level filter (e.g., `info`, `debug`, `clasp_router=trace`). Uses the standard `tracing-subscriber` `EnvFilter` syntax. |
| `LOG_FORMAT` | Set to `json` for structured JSON log output. |

## Feature Flags

Compile-time features for `cargo build --features <flag>`:

| Feature | Description |
|---------|-------------|
| `journal` | SQLite-backed state journal with replay support |
| `caps` | Ed25519 capability token validation (delegatable tokens with chain verification) |
| `registry` | Entity registry with SQLite storage |
| `rules` | JSON-based rules engine for declarative automation |
| `federation` | Router-to-router federation (hub/leaf topology) |
| `full` | All of the above |

## Examples

**Minimal development server:**

```bash
clasp-relay
```

Starts on `0.0.0.0:7330` with WebSocket, default TTLs, no auth.

**Production with auth and persistence:**

```bash
clasp-relay \
  --ws-port 7330 \
  --auth-port 7331 \
  --admin-token /etc/clasp/admin.token \
  --persist /var/lib/clasp/state.snapshot \
  --persist-interval 60 \
  --cors-origin "https://app.example.com"
```

**Multi-protocol with QUIC and MQTT:**

```bash
clasp-relay \
  --ws-port 7330 \
  --quic-port 7332 \
  --mqtt-port 1883 \
  --cert /etc/clasp/cert.pem \
  --key /etc/clasp/key.pem
```

**With app config and journal:**

```bash
clasp-relay \
  --ws-port 7330 \
  --auth-port 7331 \
  --app-config ./config/chat.json \
  --journal /var/lib/clasp/journal.db \
  --features journal
```

**Federation leaf node:**

```bash
clasp-relay \
  --ws-port 7340 \
  --federation-hub ws://hub.example.com:7330 \
  --federation-id "edge-01" \
  --federation-namespace "/lights/**" \
  --federation-namespace "/sensors/**" \
  --federation-token "$FED_TOKEN" \
  --features federation
```

**Full production deployment:**

```bash
clasp-relay \
  --ws-port 7330 \
  --auth-port 7331 \
  --health-port 7332 \
  --metrics-port 9090 \
  --admin-token /etc/clasp/admin.token \
  --app-config /etc/clasp/app.json \
  --persist /var/lib/clasp/state.snapshot \
  --journal /var/lib/clasp/journal.db \
  --trust-anchor /etc/clasp/root.pub \
  --cors-origin "https://app.example.com" \
  --drain-timeout 60 \
  --max-sessions 5000 \
  --features full
```
