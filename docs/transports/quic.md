---
title: QUIC
description: Low-latency transport with connection migration and multiplexed streams
order: 3
---

# QUIC Transport

QUIC provides lower latency than WebSocket with built-in encryption, connection migration (survives network switches), and multiplexed streams. It's the second transport the relay natively supports.

## When to Use QUIC Over WebSocket

- **Mobile apps** -- QUIC connections survive WiFi-to-cellular switches without reconnecting
- **High-throughput native apps** -- independent streams avoid head-of-line blocking
- **Fast reconnects** -- 0-RTT resumption skips the handshake on subsequent connections
- **Encryption required** -- TLS 1.3 is mandatory (no unencrypted mode)

If you need browser support or easy firewall traversal, stick with WebSocket.

## Connection URL

```
quic://host:7331    # TLS is always on (required by QUIC)
```

## Relay Configuration

QUIC is opt-in on the relay:

```bash
clasp-relay \
  --quic-port 7331 \
  --cert-path /etc/certs/relay.crt \
  --key-path /etc/certs/relay.key
```

QUIC requires TLS certificates. For development, generate self-signed:

```bash
openssl req -x509 -newkey rsa:4096 \
  -keyout key.pem -out cert.pem \
  -days 365 -nodes \
  -subj "/CN=localhost"
```

## Connecting

### Rust

```rust
use clasp_client::Clasp;

let client = Clasp::connect_to("quic://localhost:7331").await?;
```

### JavaScript (Node.js)

```javascript
import { ClaspBuilder } from '@clasp-to/core'

const client = await new ClaspBuilder('quic://localhost:7331', {
  tls: { ca: fs.readFileSync('cert.pem') }
})
```

Browser support for QUIC is limited to HTTP/3 contexts. For browsers, use WebSocket.

## Key Advantages

### Connection Migration

When a device switches networks (WiFi to cellular, different WiFi), the QUIC connection survives. The client doesn't need to reconnect or re-subscribe. This happens transparently at the transport layer.

### 0-RTT Resumption

After the first connection, subsequent connections can send data immediately without waiting for a handshake. This reduces connection setup from ~10-30ms to <5ms.

### Multiplexed Streams

Multiple independent streams share one connection. If one stream stalls (waiting for a retransmit), the others keep flowing. WebSocket over TCP has head-of-line blocking -- one lost packet stalls everything.

## Rust Transport API

```rust
use clasp_transport::quic::{QuicTransport, QuicConfig, CertVerification};

let config = QuicConfig {
    bind_addr: "0.0.0.0:0".parse()?,
    server_addr: "localhost:7331".parse()?,
    server_name: "localhost".into(),
    cert_verification: CertVerification::Custom(ca_cert),
    ..Default::default()
};

let (sender, receiver) = QuicTransport::connect(config).await?;
```

For development with self-signed certs:

```rust
let config = QuicConfig {
    cert_verification: CertVerification::Insecure,  // dev only!
    ..Default::default()
};
```

## Performance

| Metric | Typical Value |
|--------|---------------|
| Connection setup | 10-30ms (first), <5ms (0-RTT) |
| Message latency | 0.5-3ms |
| Throughput | 100,000+ msg/sec |
| Streams per connection | 100+ |

## Comparison with WebSocket

| Aspect | WebSocket | QUIC |
|--------|-----------|------|
| Underlying protocol | TCP | UDP |
| Encryption | Optional | Required (TLS 1.3) |
| Connection setup | 2-3 RTT | 1 RTT (0 with resumption) |
| Head-of-line blocking | Yes | No (per-stream) |
| Connection migration | No | Yes |
| Browser support | Full | HTTP/3 only |
| Firewall traversal | Easy | May be blocked (UDP) |

## Troubleshooting

**Connection fails** -- Check that UDP is allowed on the port. Many corporate firewalls block non-standard UDP. Also verify TLS certificates are valid.

**Slow first connection** -- First connection is ~1 RTT. Subsequent connections use 0-RTT resumption. This is normal.

**Can't connect from browser** -- QUIC in browsers is only available through HTTP/3. Use WebSocket for browser clients.

## See Also

- [WebSocket Transport](websocket.md) -- the universal fallback
- [Core Concepts: Transports](../core/transports.md) -- how transports fit into CLASP
