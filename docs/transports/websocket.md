---
title: WebSocket
description: The default transport -- reliable, bidirectional, works everywhere
order: 2
---

# WebSocket Transport

WebSocket is the default transport for CLASP. It's the only transport the relay always listens on, and every SDK connects with it out of the box.

## Why WebSocket is the Default

- Works in every browser
- Works behind proxies and firewalls (HTTP upgrade)
- Reliable and ordered delivery
- TLS support (`wss://`)
- Every language has mature WebSocket libraries

Unless you have a specific reason to use something else, use WebSocket.

## Connection URLs

```
ws://host:7330      # Unencrypted (dev/LAN)
wss://host:7330     # TLS-encrypted (production)
```

## Connecting

### JavaScript

```javascript
import { ClaspBuilder } from '@clasp-to/core'

const client = await new ClaspBuilder('ws://localhost:7330')
```

### Rust

```rust
use clasp_client::Clasp;

let client = Clasp::connect_to("ws://localhost:7330").await?;
```

### Python

```python
from clasp import ClaspBuilder

client = await ClaspBuilder("ws://localhost:7330")
```

## Relay Configuration

The relay listens on WebSocket by default on port 7330:

```bash
clasp-relay                           # ws://0.0.0.0:7330
clasp-relay --ws-port 8080            # ws://0.0.0.0:8080
clasp-relay --host 127.0.0.1          # localhost only
```

### With TLS

```bash
clasp-relay \
  --cert-path /etc/certs/relay.crt \
  --key-path /etc/certs/relay.key
```

Clients connect with `wss://` when TLS is enabled.

## How It Works

1. Client opens a TCP connection to the relay
2. HTTP upgrade handshake promotes it to a WebSocket connection
3. CLASP binary frames are sent as WebSocket binary messages (not text)
4. Ping/pong frames keep the connection alive
5. Either side can close gracefully

```
┌────────────────────────────────┐
│ WebSocket Binary Frame         │
│ ┌────────────────────────────┐ │
│ │ CLASP Binary Frame         │ │
│ │ (same format regardless    │ │
│ │  of transport)             │ │
│ └────────────────────────────┘ │
└────────────────────────────────┘
```

One CLASP frame per WebSocket message. No additional framing needed -- WebSocket handles message boundaries.

## Rust Transport API

```rust
use clasp_transport::websocket::{WebSocketTransport, WebSocketConfig};

// Client side
let config = WebSocketConfig {
    url: "ws://localhost:7330".into(),
    ..Default::default()
};
let (sender, receiver) = WebSocketTransport::connect(config).await?;

// Server side
let server = WebSocketServer::bind("0.0.0.0:7330").await?;
loop {
    let (sender, receiver, addr) = server.accept().await?;
    tokio::spawn(handle_client(sender, receiver, addr));
}
```

## WASM (Browser)

In browser/WASM environments, use the `wasm-websocket` feature flag instead of `websocket`. This uses the browser's native `WebSocket` API under the hood:

```rust
use clasp_transport::wasm_websocket::{WasmWebSocketConfig, WasmWebSocketTransport};

let config = WasmWebSocketConfig {
    url: "ws://localhost:7330".into(),
};
let (sender, receiver) = WasmWebSocketTransport::connect(config).await?;
```

The API is the same `TransportSender`/`TransportReceiver` trait -- your application code doesn't change.

## Proxy Configuration

### nginx

```nginx
location / {
    proxy_pass http://localhost:7330;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    proxy_set_header Host $host;
    proxy_read_timeout 3600s;
}
```

### Caddy

```
clasp.example.com {
    reverse_proxy localhost:7330
}
```

Caddy handles WebSocket upgrade automatically.

## Performance

| Metric | Typical Value |
|--------|---------------|
| Connection setup | 5-20ms (local), 50-200ms (remote) |
| Message latency | 1-5ms (local) |
| Throughput | 50,000+ msg/sec per connection |
| Memory per connection | ~10KB |

## Troubleshooting

**Connection refused** -- Relay isn't running, or wrong port. Check `clasp-relay` is listening on the expected port.

**Connection closed immediately** -- TLS mismatch. If relay has TLS enabled, client must use `wss://`. If not, use `ws://`.

**Behind a proxy, connection drops after 60s** -- Proxy is timing out idle connections. Increase the read timeout (`proxy_read_timeout` in nginx) or ensure ping/pong is working.

**CORS errors in browser** -- Set `--cors-origin` on the relay to allow your frontend's origin, or use `*` for development.

## See Also

- [QUIC Transport](quic.md) -- lower latency alternative for native clients
- [Core Concepts: Transports](../core/transports.md) -- how transports fit into CLASP
