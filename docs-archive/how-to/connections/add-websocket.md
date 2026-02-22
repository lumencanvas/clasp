---
title: "Add WebSocket Bridge"
description: "Add a WebSocket bridge for generic JSON WebSocket clients."
section: how-to
order: 7
---
# Add WebSocket Bridge

Add a WebSocket bridge for generic JSON WebSocket clients.

## When to Use

Use the WebSocket bridge when:
- Connecting apps that speak generic WebSocket JSON (not native CLASP)
- Bridging existing WebSocket applications
- Creating a JSON-friendly API for web clients

For native CLASP clients, connect directly to the router instead.

## Start WebSocket Bridge

### CLI

```bash
clasp websocket --bind 0.0.0.0:8080 --format json
```

### Options

| Option | Description |
|--------|-------------|
| `--format json` | JSON message format |
| `--format msgpack` | MessagePack format |
| `--prefix /ws` | Address prefix |

## Message Format

### JSON Format

**Set value:**
```json
{
  "type": "set",
  "address": "/lights/brightness",
  "value": 0.8
}
```

**Get value:**
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
  "value": 0.8
}
```

**Subscribe:**
```json
{
  "type": "subscribe",
  "pattern": "/lights/**"
}
```

Updates arrive as:
```json
{
  "type": "update",
  "address": "/lights/1/brightness",
  "value": 0.5
}
```

**Emit event:**
```json
{
  "type": "emit",
  "address": "/cue/fire",
  "payload": {"id": "intro"}
}
```

## Browser Example

```javascript
const ws = new WebSocket('ws://localhost:8080');

ws.onopen = () => {
  // Subscribe
  ws.send(JSON.stringify({
    type: 'subscribe',
    pattern: '/lights/**'
  }));

  // Set value
  ws.send(JSON.stringify({
    type: 'set',
    address: '/lights/1/brightness',
    value: 0.75
  }));
};

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);

  if (msg.type === 'update') {
    console.log(`${msg.address} = ${msg.value}`);
  }
};
```

## Socket.IO (Planned)

Socket.IO support is planned:

```bash
clasp socketio --bind 0.0.0.0:8080
```

```javascript
const socket = io('http://localhost:8080');

socket.on('clasp:update', (data) => {
  console.log(data.address, data.value);
});

socket.emit('clasp:set', {
  address: '/lights/brightness',
  value: 0.8
});
```

## Native CLASP vs WebSocket Bridge

| Feature | Native CLASP | WebSocket Bridge |
|---------|--------------|------------------|
| Protocol | Binary CLASP | JSON/MessagePack |
| Efficiency | Higher | Lower |
| Browser | Supported | Supported |
| Existing apps | Needs client lib | Works with any WS |

Use native CLASP (`@clasp-to/core`) when possible. Use the bridge for:
- Legacy applications
- Simple integrations
- Testing with generic tools

## Troubleshooting

### Messages not received

- Verify JSON format is valid
- Check message type spelling
- Ensure subscription pattern matches

### Connection closes

- Check for ping/pong timeout
- Verify no other service on the port

## Next Steps

- [JavaScript Library](../installation/javascript-library.md) — Native CLASP client
- [Connect a Client](connect-client.md)
