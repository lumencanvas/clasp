---
title: "clasp http"
description: "Start an HTTP/REST bridge."
section: reference
order: 1
---
# clasp http

Start an HTTP/REST bridge.

## Synopsis

```
clasp http [OPTIONS]
```

## Description

Creates an HTTP REST API bridge for CLASP. Exposes CLASP state via HTTP endpoints and accepts HTTP requests to modify state.

## Options

### HTTP Server

```
--port <PORT>
    HTTP server port [default: 3000]

--bind <ADDRESS>
    Address to bind to [default: 0.0.0.0]

--tls
    Enable HTTPS

--tls-cert <PATH>
    TLS certificate file

--tls-key <PATH>
    TLS private key file
```

### CLASP Connection

```
--router <URL>
    CLASP router URL [default: ws://localhost:7330]

--token <TOKEN>
    Authentication token

--name <NAME>
    Client name [default: http-bridge]
```

### CORS

```
--cors
    Enable CORS

--cors-origin <ORIGIN>
    Allowed origins (can be repeated, or * for all)

--cors-methods <METHODS>
    Allowed methods [default: GET,POST,PUT,DELETE]
```

### Authentication

```
--api-key <KEY>
    Require API key in X-API-Key header

--basic-auth <USER:PASS>
    Require HTTP Basic authentication
```

### Options

```
--prefix <PREFIX>
    URL path prefix [default: /api]

--sse
    Enable Server-Sent Events endpoint

--websocket
    Enable WebSocket endpoint (proxy to router)
```

### Other

```
-c, --config <PATH>
    Configuration file

-v, --verbose
    Verbose output

-h, --help
    Print help
```

## Endpoints

### GET /api/state/{address}

Get value at address.

```bash
curl http://localhost:3000/api/state/sensors/temp

# Response:
{"address": "/sensors/temp", "value": 23.5}
```

### PUT /api/state/{address}

Set value at address.

```bash
curl -X PUT http://localhost:3000/api/state/control/brightness \
  -H "Content-Type: application/json" \
  -d '{"value": 128}'

# Response:
{"success": true}
```

### DELETE /api/state/{address}

Delete value at address.

```bash
curl -X DELETE http://localhost:3000/api/state/temp/data

# Response:
{"success": true}
```

### POST /api/event/{address}

Emit event.

```bash
curl -X POST http://localhost:3000/api/event/button/pressed \
  -H "Content-Type: application/json" \
  -d '{"button": 1}'

# Response:
{"success": true}
```

### GET /api/list/{pattern}

List addresses matching pattern.

```bash
curl http://localhost:3000/api/list/sensors/**

# Response:
{"addresses": ["/sensors/temp", "/sensors/humidity"]}
```

### GET /api/subscribe/{pattern} (SSE)

Subscribe to changes via Server-Sent Events.

```bash
curl http://localhost:3000/api/subscribe/sensors/**

# Stream:
data: {"address": "/sensors/temp", "value": 23.5}

data: {"address": "/sensors/temp", "value": 23.6}
```

### POST /api/bundle

Execute atomic bundle.

```bash
curl -X POST http://localhost:3000/api/bundle \
  -H "Content-Type: application/json" \
  -d '{
    "operations": [
      {"set": ["/lights/1", 255]},
      {"set": ["/lights/2", 128]}
    ]
  }'

# Response:
{"success": true}
```

## Examples

### Basic Server

```bash
clasp http --port 3000
```

### With CORS

```bash
clasp http --port 3000 --cors --cors-origin "*"
```

### With API Key

```bash
clasp http --port 3000 --api-key "secret-key"

# Client must include header:
curl -H "X-API-Key: secret-key" http://localhost:3000/api/state/sensors/temp
```

### With HTTPS

```bash
clasp http --port 3000 \
  --tls \
  --tls-cert /path/to/cert.pem \
  --tls-key /path/to/key.pem
```

### Production Setup

```bash
clasp http --port 3000 \
  --tls \
  --tls-cert /etc/letsencrypt/live/api.example.com/fullchain.pem \
  --tls-key /etc/letsencrypt/live/api.example.com/privkey.pem \
  --cors \
  --cors-origin "https://app.example.com" \
  --api-key "$API_KEY" \
  --sse
```

## Configuration File

```yaml
# http-bridge.yaml
http:
  port: 3000
  bind: "0.0.0.0"
  tls:
    enabled: true
    cert: /path/to/cert.pem
    key: /path/to/key.pem

  cors:
    enabled: true
    origins:
      - "https://app.example.com"
      - "https://admin.example.com"
    methods: ["GET", "POST", "PUT", "DELETE"]

  auth:
    api_key: "${API_KEY}"
    # Or: basic_auth: "user:password"

  features:
    sse: true
    websocket: false

  prefix: "/api"

clasp:
  router: "ws://localhost:7330"
  name: "http-bridge"
```

## JavaScript Client Example

```javascript
// GET value
const response = await fetch('http://localhost:3000/api/state/sensors/temp');
const { value } = await response.json();

// SET value
await fetch('http://localhost:3000/api/state/control/brightness', {
  method: 'PUT',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ value: 128 })
});

// Subscribe with SSE
const events = new EventSource('http://localhost:3000/api/subscribe/sensors/**');
events.onmessage = (e) => {
  const { address, value } = JSON.parse(e.data);
  console.log(`${address}: ${value}`);
};
```

## See Also

- [Add HTTP](../../how-to/connections/add-http.md)
- [HTTP Bridge Reference](../bridges/http.md)
- [Software Integration](../../use-cases/software-integration.md)
