# HTTP Bridge

REST API interface for CLASP.

## Overview

The HTTP bridge exposes CLASP state and operations via a RESTful HTTP API, enabling integration with web applications, scripts, and services that use HTTP.

## Endpoints

### GET /api/state/{address}

Retrieve value at an address.

**Request:**
```http
GET /api/state/sensors/temperature HTTP/1.1
Host: localhost:3000
```

**Response:**
```json
{
  "address": "/sensors/temperature",
  "value": 23.5,
  "timestamp": 1704067200000
}
```

**Status Codes:**
- 200: Success
- 404: Address not found
- 500: Server error

### PUT /api/state/{address}

Set value at an address.

**Request:**
```http
PUT /api/state/control/brightness HTTP/1.1
Host: localhost:3000
Content-Type: application/json

{"value": 128}
```

**Response:**
```json
{
  "success": true
}
```

**Status Codes:**
- 200: Success
- 400: Invalid request
- 403: Permission denied
- 500: Server error

### DELETE /api/state/{address}

Delete value at an address.

**Request:**
```http
DELETE /api/state/temp/data HTTP/1.1
Host: localhost:3000
```

**Response:**
```json
{
  "success": true
}
```

### POST /api/event/{address}

Emit an event.

**Request:**
```http
POST /api/event/button/pressed HTTP/1.1
Host: localhost:3000
Content-Type: application/json

{"button": 1, "timestamp": 1704067200000}
```

**Response:**
```json
{
  "success": true
}
```

### GET /api/list/{pattern}

List addresses matching a pattern.

**Request:**
```http
GET /api/list/sensors/** HTTP/1.1
Host: localhost:3000
```

**Response:**
```json
{
  "addresses": [
    "/sensors/temperature",
    "/sensors/humidity",
    "/sensors/room1/temp"
  ]
}
```

### POST /api/bundle

Execute atomic bundle of operations.

**Request:**
```http
POST /api/bundle HTTP/1.1
Host: localhost:3000
Content-Type: application/json

{
  "operations": [
    {"set": ["/lights/1", 255]},
    {"set": ["/lights/2", 128]},
    {"emit": ["/cue/fired", {"cue": 1}]}
  ],
  "timestamp": 1704067200000
}
```

**Response:**
```json
{
  "success": true
}
```

### GET /api/subscribe/{pattern}

Subscribe to changes via Server-Sent Events.

**Request:**
```http
GET /api/subscribe/sensors/** HTTP/1.1
Host: localhost:3000
Accept: text/event-stream
```

**Response (SSE stream):**
```
data: {"address":"/sensors/temperature","value":23.5}

data: {"address":"/sensors/humidity","value":65}

data: {"address":"/sensors/temperature","value":23.6}
```

## Authentication

### API Key

Include API key in header:

```http
GET /api/state/sensors/temp HTTP/1.1
Host: localhost:3000
X-API-Key: your-api-key
```

### Bearer Token

Use JWT token:

```http
GET /api/state/sensors/temp HTTP/1.1
Host: localhost:3000
Authorization: Bearer eyJhbGciOi...
```

### Basic Auth

HTTP Basic authentication:

```http
GET /api/state/sensors/temp HTTP/1.1
Host: localhost:3000
Authorization: Basic dXNlcjpwYXNzd29yZA==
```

## CORS

Enable CORS for browser access:

```yaml
http:
  cors:
    enabled: true
    origins:
      - "https://app.example.com"
      - "http://localhost:3000"
    methods: ["GET", "POST", "PUT", "DELETE"]
    headers: ["Content-Type", "X-API-Key"]
```

## Configuration

### CLI

```bash
clasp http --port 3000 --cors --api-key "secret"
```

### Configuration File

```yaml
http:
  port: 3000
  bind: "0.0.0.0"

  tls:
    enabled: false
    cert: /path/to/cert.pem
    key: /path/to/key.pem

  auth:
    api_key: "${API_KEY}"
    # Or:
    # basic: "user:password"
    # jwt_secret: "${JWT_SECRET}"

  cors:
    enabled: true
    origins: ["*"]

  features:
    sse: true
    websocket: false

  prefix: "/api"

clasp:
  router: "ws://localhost:7330"
```

### Rust API

```rust
use clasp_bridge::http::{HttpBridge, HttpConfig};

let config = HttpConfig {
    bind_addr: "0.0.0.0:3000".parse()?,
    api_key: Some("secret".into()),
    cors_enabled: true,
    sse_enabled: true,
};

let bridge = HttpBridge::new(client, config).await?;
```

## Client Examples

### JavaScript (Fetch)

```javascript
// GET
const response = await fetch('http://localhost:3000/api/state/sensors/temp');
const { value } = await response.json();

// SET
await fetch('http://localhost:3000/api/state/control/brightness', {
  method: 'PUT',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ value: 128 })
});

// SSE Subscribe
const events = new EventSource('http://localhost:3000/api/subscribe/sensors/**');
events.onmessage = (e) => {
  const { address, value } = JSON.parse(e.data);
  console.log(`${address}: ${value}`);
};
```

### Python (requests)

```python
import requests

# GET
response = requests.get('http://localhost:3000/api/state/sensors/temp')
value = response.json()['value']

# SET
requests.put(
    'http://localhost:3000/api/state/control/brightness',
    json={'value': 128}
)

# SSE (requires sseclient)
import sseclient

response = requests.get(
    'http://localhost:3000/api/subscribe/sensors/**',
    stream=True
)
client = sseclient.SSEClient(response)
for event in client.events():
    data = json.loads(event.data)
    print(f"{data['address']}: {data['value']}")
```

### cURL

```bash
# GET
curl http://localhost:3000/api/state/sensors/temp

# SET
curl -X PUT http://localhost:3000/api/state/control/brightness \
  -H "Content-Type: application/json" \
  -d '{"value": 128}'

# With API key
curl -H "X-API-Key: secret" http://localhost:3000/api/state/sensors/temp
```

## See Also

- [Add HTTP](../../how-to/connections/add-http.md)
- [clasp http CLI](../cli/clasp-http.md)
- [Software Integration](../../use-cases/software-integration.md)
