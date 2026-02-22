---
title: WebSocket Bridge
description: Generic WebSocket JSON bridge for CLASP
order: 9
---

# WebSocket Bridge

Bridge generic WebSocket connections using JSON messages. The WebSocket bridge is designed for web apps and services that want persistent, bidirectional communication with CLASP without using the CLASP binary protocol or an SDK. Clients send and receive simple JSON messages over a standard WebSocket connection.

## Quick Start

```bash
clasp bridge websocket --port 9000 --target ws://localhost:7330
```

Connect from any WebSocket client:

```javascript
const ws = new WebSocket('ws://localhost:9000');

ws.onopen = () => {
  // Subscribe to changes
  ws.send(JSON.stringify({
    type: 'subscribe',
    pattern: '/lights/*'
  }));

  // Set a value
  ws.send(JSON.stringify({
    type: 'set',
    address: '/lights/brightness',
    value: 0.8
  }));
};

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  console.log(msg.address, msg.value);
};
```

## Protocol

Messages are JSON objects sent as WebSocket text frames. Every message has a `type` field that determines its structure.

### Client-to-Bridge Messages

**set** -- set a Param value:

```json
{
  "type": "set",
  "address": "/lights/brightness",
  "value": 0.8
}
```

**get** -- request the current value at an address:

```json
{
  "type": "get",
  "address": "/lights/brightness"
}
```

Response:

```json
{
  "type": "value",
  "address": "/lights/brightness",
  "value": 0.8,
  "revision": 42
}
```

**emit** -- emit an Event signal (not stored in state):

```json
{
  "type": "emit",
  "address": "/cue/go",
  "value": {"scene": "intro"}
}
```

**subscribe** -- subscribe to changes matching a pattern:

```json
{
  "type": "subscribe",
  "pattern": "/lights/*"
}
```

**unsubscribe** -- remove a subscription:

```json
{
  "type": "unsubscribe",
  "pattern": "/lights/*"
}
```

**snapshot** -- request a snapshot of all current state:

```json
{
  "type": "snapshot"
}
```

Response:

```json
{
  "type": "snapshot",
  "state": {
    "/lights/brightness": 0.8,
    "/lights/color": [255, 0, 128],
    "/mixer/ch/1/fader": 0.6
  }
}
```

Filtered snapshot:

```json
{
  "type": "snapshot",
  "prefix": "/lights"
}
```

### Bridge-to-Client Messages

**value** -- delivered when a subscribed address changes:

```json
{
  "type": "value",
  "address": "/lights/brightness",
  "value": 0.8,
  "revision": 43
}
```

**event** -- delivered when an Event signal matches a subscription:

```json
{
  "type": "event",
  "address": "/cue/go",
  "value": {"scene": "intro"}
}
```

**error** -- delivered when a request fails:

```json
{
  "type": "error",
  "message": "invalid address format",
  "request_type": "set"
}
```

## Message Types

| Type | Direction | Description |
|---|---|---|
| `set` | Client to bridge | Set a Param value at an address |
| `get` | Client to bridge | Request the current value |
| `emit` | Client to bridge | Emit a one-shot Event signal |
| `subscribe` | Client to bridge | Subscribe to address pattern changes |
| `unsubscribe` | Client to bridge | Remove a subscription |
| `snapshot` | Client to bridge | Request full or filtered state snapshot |
| `value` | Bridge to client | Param value update (from subscription or get) |
| `event` | Bridge to client | Event signal delivery |
| `snapshot` | Bridge to client | State snapshot response |
| `error` | Bridge to client | Error response |

## Configuration

| Option | CLI Flag | Default | Description |
|---|---|---|---|
| Listen port | `--port` | `9000` | WebSocket server port |
| Target router | `--target` | `ws://localhost:7330` | WebSocket URL of the CLASP router |
| Bind address | `--bind` | `0.0.0.0` | Network interface to bind |
| Auth token | `--token` | -- | Require token in connection URL query string |

## Use Cases

**Simple web integrations**: connect a web page to CLASP with plain JavaScript and no build step. No SDK installation required.

**Prototyping**: quickly test CLASP interactions from a browser console or a WebSocket testing tool like websocat.

**Services without CLASP SDK**: connect services written in languages without a CLASP SDK. Any language with a WebSocket library can use this bridge.

**Lightweight embedded clients**: microcontrollers or constrained devices that support WebSocket but cannot run the full CLASP binary protocol.

## Differences from the CLASP WebSocket Transport

The CLASP router itself uses WebSocket as a transport for the binary CLASP protocol. This bridge is different:

| | CLASP WebSocket Transport | WebSocket JSON Bridge |
|---|---|---|
| Protocol | CLASP binary frames | JSON text messages |
| Port | Router port (7330) | Bridge port (9000) |
| SDK required | Yes | No |
| Full protocol support | Yes | Subset (set, get, emit, subscribe) |
| Performance | Higher (binary) | Lower (JSON parsing) |

Use the CLASP WebSocket transport (via an SDK) for production applications. Use the WebSocket JSON bridge for quick integrations, prototyping, and services that cannot use an SDK.

## Troubleshooting

**Connection closes immediately**
- Verify the bridge is running on the expected port: `clasp bridge websocket --port 9000`
- If `--token` is set, include it in the connection URL: `ws://localhost:9000?token=your-token`

**No messages received after subscribe**
- Check the subscription pattern. Patterns use glob syntax: `*` matches one level, `**` matches multiple levels.
- Verify that another client is publishing to the subscribed addresses.

**JSON parse errors**
- Ensure messages are valid JSON. Common mistake: using single quotes instead of double quotes.
- Every message must have a `type` field.

## Next Steps

- [HTTP Bridge](http.md) -- REST API for one-shot requests
- [Protocol Bridges Overview](../protocols/README.md) -- all 8 protocol bridges
- [JavaScript SDK](../sdk/javascript.md) -- for production web applications, use the SDK instead
