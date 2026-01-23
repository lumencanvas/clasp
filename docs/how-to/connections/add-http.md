# Add HTTP

Add REST API access to CLASP for web integrations and webhooks.

## Prerequisites

- Running CLASP router

## Start HTTP Bridge

### CLI

```bash
clasp http --bind 0.0.0.0:3000
```

### Desktop App

1. Click **Add Protocol**
2. Select **HTTP REST API**
3. Configure port (default: 3000)
4. Click **Start**

## API Endpoints

### Set Value

```bash
curl -X POST http://localhost:3000/api/set \
  -H "Content-Type: application/json" \
  -d '{"address": "/lights/brightness", "value": 0.8}'
```

### Get Value

```bash
curl http://localhost:3000/api/get?address=/lights/brightness
```

Response:
```json
{"value": 0.8, "revision": 42}
```

### Emit Event

```bash
curl -X POST http://localhost:3000/api/emit \
  -H "Content-Type: application/json" \
  -d '{"address": "/cue/fire", "payload": {"id": "intro"}}'
```

### Query State

```bash
curl "http://localhost:3000/api/query?pattern=/lights/**"
```

Response:
```json
[
  {"address": "/lights/1/brightness", "value": 0.8},
  {"address": "/lights/2/brightness", "value": 0.5}
]
```

## Custom Endpoints

Create custom REST endpoints that map to CLASP:

### Desktop App

1. In "REST API" section, click "Add Endpoint"
2. Configure:
   - Path: `/lights/on`
   - Method: POST
   - Action: SET `/lights/power` = true

### Configuration File

```yaml
http:
  endpoints:
    - path: /lights/on
      method: POST
      action:
        type: set
        address: /lights/power
        value: true

    - path: /lights/brightness/:value
      method: PUT
      action:
        type: set
        address: /lights/brightness
        value: "$params.value"

    - path: /cue/:id
      method: POST
      action:
        type: emit
        address: /cue/fire
        payload:
          id: "$params.id"
```

## From CLASP to HTTP (Webhooks)

Trigger HTTP requests when CLASP values change:

```bash
clasp http --bind 0.0.0.0:3000 \
  --webhook "/alerts/**" "https://webhook.example.com/notify"
```

```javascript
// When this is set...
await client.emit('/alerts/high-temp', { sensor: 'room1' });

// ...webhook.example.com receives POST with payload
```

## Authentication

### API Key

```bash
clasp http --bind 0.0.0.0:3000 --api-key "secret123"
```

Clients must include header:
```bash
curl -H "X-API-Key: secret123" http://localhost:3000/api/get?address=/path
```

### Bearer Token

```bash
clasp http --bind 0.0.0.0:3000 --require-auth
```

Use CLASP capability tokens:
```bash
curl -H "Authorization: Bearer eyJhbGci..." http://localhost:3000/api/get
```

## CORS

CORS is enabled by default for browser access. Customize:

```bash
clasp http --bind 0.0.0.0:3000 \
  --cors-origin "https://myapp.com" \
  --cors-origin "https://admin.myapp.com"
```

## Server-Sent Events (SSE)

Subscribe to changes via SSE:

```javascript
const events = new EventSource('http://localhost:3000/api/events?pattern=/lights/**');

events.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log(data.address, data.value);
};
```

## Example: Smart Home Integration

```bash
# Start HTTP bridge
clasp http --bind 0.0.0.0:3000
```

```javascript
// Web interface
async function setLight(id, brightness) {
  await fetch('http://localhost:3000/api/set', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      address: `/lights/${id}/brightness`,
      value: brightness
    })
  });
}

// Webhook from external service
// POST to http://localhost:3000/api/emit
// {"address": "/external/doorbell", "payload": {"event": "ring"}}
```

## Troubleshooting

### CORS errors

Add your origin:
```bash
clasp http --cors-origin "http://localhost:8080"
```

### 404 Not Found

Check endpoint path matches exactly. Paths are case-sensitive.

### Connection refused

Verify HTTP bridge is running and port is accessible.

## Next Steps

- [Add WebSocket](add-websocket.md)
- [HTTP Bridge Reference](../../reference/bridges/http.md)
