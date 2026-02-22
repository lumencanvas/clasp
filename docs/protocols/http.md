---
title: HTTP Bridge
description: REST API bridge for CLASP
order: 8
---

# HTTP Bridge

Bridge HTTP REST requests to CLASP. The HTTP bridge exposes a REST API for getting and setting CLASP state from any HTTP client -- curl, browser fetch, webhooks, or backend services. No CLASP SDK required.

## Quick Start

```bash
clasp http --port 8080 --target ws://localhost:7330
```

The bridge starts an HTTP server on port 8080 and connects to the CLASP router. You can immediately interact with CLASP state:

```bash
# Set a value
curl -X PUT http://localhost:8080/v1/state/lights/brightness \
  -H "Content-Type: application/json" \
  -d '{"value": 0.8}'

# Get a value
curl http://localhost:8080/v1/state/lights/brightness

# Emit an event
curl -X POST http://localhost:8080/v1/emit/cue/go \
  -H "Content-Type: application/json" \
  -d '{"value": {"scene": "intro"}}'

# Get all state
curl http://localhost:8080/v1/snapshot
```

## Endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/v1/state/{address}` | Get the current value at an address |
| `PUT` | `/v1/state/{address}` | Set a Param value at an address |
| `POST` | `/v1/emit/{address}` | Emit an Event signal |
| `GET` | `/v1/snapshot` | Get the full state tree |
| `GET` | `/v1/snapshot/{prefix}` | Get state under a prefix |
| `DELETE` | `/v1/state/{address}` | Remove a value from state |

### GET /v1/state/{address}

Returns the current value at the given address:

```bash
curl http://localhost:8080/v1/state/lights/brightness
```

```json
{
  "address": "/lights/brightness",
  "value": 0.8,
  "revision": 42,
  "timestamp": "2026-02-21T12:00:00Z"
}
```

Returns `404` if the address has no value.

### PUT /v1/state/{address}

Sets a Param value:

```bash
curl -X PUT http://localhost:8080/v1/state/lights/brightness \
  -H "Content-Type: application/json" \
  -d '{"value": 0.5}'
```

```json
{
  "address": "/lights/brightness",
  "value": 0.5,
  "revision": 43
}
```

### POST /v1/emit/{address}

Emits an Event signal. Events are not stored in state:

```bash
curl -X POST http://localhost:8080/v1/emit/cue/go \
  -H "Content-Type: application/json" \
  -d '{"value": {"scene": "finale"}}'
```

```json
{
  "address": "/cue/go",
  "status": "emitted"
}
```

### GET /v1/snapshot

Returns the full state tree as a JSON object:

```bash
curl http://localhost:8080/v1/snapshot
```

```json
{
  "/lights/brightness": 0.8,
  "/lights/color": [255, 0, 128],
  "/mixer/ch/1/fader": 0.6
}
```

Filter by prefix:

```bash
curl http://localhost:8080/v1/snapshot/lights
```

```json
{
  "/lights/brightness": 0.8,
  "/lights/color": [255, 0, 128]
}
```

## Request/Response Format

All request and response bodies use JSON with `Content-Type: application/json`.

**Value types** in request bodies:

| JSON Type | CLASP Type |
|---|---|
| number (integer) | i64 |
| number (float) | f64 |
| string | string |
| boolean | bool |
| array | array |
| object | Map |
| null | null (deletes the value) |

## Configuration

| Option | CLI Flag | Default | Description |
|---|---|---|---|
| Listen port | `--port` | `8080` | HTTP server port |
| Target router | `--target` | `ws://localhost:7330` | WebSocket URL of the CLASP router |
| Bind address | `--bind` | `0.0.0.0` | Network interface to bind |
| CORS origins | `--cors` | `*` | Allowed CORS origins |
| Auth token | `--token` | -- | Require Bearer token for requests |

## Use Cases

**Webhooks**: receive HTTP webhooks from third-party services (GitHub, Stripe, IFTTT) and translate them into CLASP events.

**REST integrations**: connect backend services written in any language to CLASP without installing an SDK.

**One-shot queries**: quickly inspect or set state from the command line with curl.

**Dashboards**: fetch state snapshots for monitoring dashboards that poll rather than subscribe.

## Troubleshooting

**CORS errors in browser**
- Set `--cors` to the origin of your web app: `clasp http --port 8080 --cors "http://localhost:5173"`
- For development, `--cors "*"` allows all origins

**404 on GET**
- The address has no value in state. Verify the address exists by checking `/v1/snapshot`

**Connection refused to target router**
- Ensure the relay is running at the URL specified by `--target`

## Next Steps

- [WebSocket Bridge](websocket.md) -- persistent connections with JSON messages
- [Protocol Bridges Overview](../protocols/README.md) -- all 8 protocol bridges
- Bridge Configuration -- address mapping and auth configuration
