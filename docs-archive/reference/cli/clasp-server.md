---
title: "clasp server"
description: "Start a CLASP router."
section: reference
order: 5
---
# clasp server

Start a CLASP router.

## Synopsis

```
clasp server [OPTIONS]
```

## Description

Starts a CLASP router that clients can connect to. The router handles message routing, state management, and subscription matching.

## Options

### Network

```
--port <PORT>
    WebSocket port to listen on [default: 7330]

--bind <ADDRESS>
    Address to bind to [default: 0.0.0.0]

--quic
    Enable QUIC transport (requires TLS)

--quic-port <PORT>
    QUIC port [default: same as --port]

--udp
    Enable UDP transport

--udp-port <PORT>
    UDP port [default: 7331]
```

### TLS

```
--tls
    Enable TLS (WSS)

--tls-cert <PATH>
    Path to TLS certificate file

--tls-key <PATH>
    Path to TLS private key file
```

### Discovery

```
--mdns
    Enable mDNS advertisement

--mdns-name <NAME>
    Name to advertise via mDNS [default: hostname]

--discovery-port <PORT>
    UDP discovery port [default: 7331]
```

### Security

```
--require-auth
    Require authentication token

--token-secret <SECRET>
    Secret for JWT token validation

--token-public-key <PATH>
    Public key file for RS256 tokens

--pairing
    Enable pairing mode

--pairing-timeout <SECONDS>
    Pairing PIN timeout [default: 300]
```

### Limits

```
--max-connections <N>
    Maximum client connections [default: 10000]

--max-message-size <BYTES>
    Maximum message size [default: 65536]

--max-subscriptions <N>
    Maximum subscriptions per client [default: 1000]
```

### Persistence

```
--persist
    Enable state persistence

--persist-path <PATH>
    Persistence database path [default: ./clasp-state.db]

--persist-interval <SECONDS>
    Sync interval [default: 5]
```

### Logging

```
--log-level <LEVEL>
    Log level: error, warn, info, debug, trace [default: info]

--log-format <FORMAT>
    Log format: text, json [default: text]

--log-file <PATH>
    Log to file instead of stdout
```

### Metrics

```
--metrics
    Enable Prometheus metrics

--metrics-port <PORT>
    Metrics HTTP port [default: 9090]
```

### Other

```
-c, --config <PATH>
    Configuration file path

-h, --help
    Print help

-V, --version
    Print version
```

## Examples

### Basic Server

```bash
clasp server
```

Starts router on port 7330.

### With TLS

```bash
clasp server --tls \
  --tls-cert /path/to/cert.pem \
  --tls-key /path/to/key.pem
```

### With Discovery

```bash
clasp server --mdns --mdns-name "Studio Router"
```

### With Authentication

```bash
clasp server --require-auth --token-secret "your-256-bit-secret"
```

### With Persistence

```bash
clasp server --persist --persist-path /var/lib/clasp/state.db
```

### Production Setup

```bash
clasp server \
  --port 7330 \
  --tls \
  --tls-cert /etc/letsencrypt/live/example.com/fullchain.pem \
  --tls-key /etc/letsencrypt/live/example.com/privkey.pem \
  --require-auth \
  --token-secret "$TOKEN_SECRET" \
  --persist \
  --persist-path /var/lib/clasp/state.db \
  --metrics \
  --log-level info \
  --log-format json
```

## Configuration File

```yaml
# clasp.yaml
server:
  port: 7330
  bind: "0.0.0.0"

  tls:
    enabled: true
    cert: /path/to/cert.pem
    key: /path/to/key.pem

  discovery:
    mdns:
      enabled: true
      name: "Studio Router"

  security:
    require_auth: true
    token_secret: ${TOKEN_SECRET}

  limits:
    max_connections: 10000
    max_message_size: 65536

  persistence:
    enabled: true
    path: /var/lib/clasp/state.db
    sync_interval: 5

  logging:
    level: info
    format: json

  metrics:
    enabled: true
    port: 9090
```

Use with:

```bash
clasp server -c clasp.yaml
```

## Environment Variables

```bash
CLASP_PORT=7330
CLASP_BIND=0.0.0.0
CLASP_TLS_CERT=/path/to/cert.pem
CLASP_TLS_KEY=/path/to/key.pem
CLASP_TOKEN_SECRET=your-secret
CLASP_LOG_LEVEL=info
```

## Signals

- `SIGTERM` / `SIGINT`: Graceful shutdown
- `SIGHUP`: Reload configuration (if supported)

## See Also

- [Start Router](../../how-to/connections/start-router.md)
- [Enable TLS](../../how-to/security/enable-tls.md)
- [Cloud Deployment](../../use-cases/cloud-deployment.md)
