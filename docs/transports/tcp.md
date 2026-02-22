---
title: TCP
description: Raw TCP transport for server-to-server and LAN communication
order: 4
---

# TCP Transport

TCP transport provides reliable, ordered delivery over raw TCP sockets. It's best for server-to-server communication on a trusted LAN where you don't need WebSocket's HTTP upgrade overhead.

## When to Use TCP

- Server-to-server communication on a LAN
- Embedding `clasp-router` in your own Rust binary
- Environments where WebSocket overhead isn't justified
- Intra-process or localhost connections

For most other cases, WebSocket is a better choice -- it works through proxies and firewalls, which raw TCP does not.

## Important: The Relay Does Not Listen on TCP

The `clasp-relay` binary only listens on WebSocket and optionally QUIC. It does **not** accept raw TCP connections.

To accept TCP connections, embed the `clasp-router` crate in your own Rust binary:

```rust
use clasp_router::Router;
use clasp_transport::tcp::{TcpServer, TcpConfig};

let router = Router::new();

// Accept TCP connections
let mut server = TcpServer::bind("0.0.0.0:7340").await?;
loop {
    let (sender, receiver, addr) = server.accept().await?;
    router.add_client(sender, receiver, addr).await;
}
```

## Rust Transport API

### Client

```rust
use clasp_transport::tcp::{TcpTransport, TcpConfig};

let config = TcpConfig {
    addr: "192.168.1.100:7340".parse()?,
    ..Default::default()
};

let (sender, receiver) = TcpTransport::connect(config).await?;

sender.send(clasp_frame).await?;
```

### Server

```rust
use clasp_transport::tcp::TcpServer;

let mut server = TcpServer::bind("0.0.0.0:7340").await?;
let (sender, receiver, addr) = server.accept().await?;
```

## Frame Delimiting

Unlike WebSocket (which has built-in message boundaries), raw TCP is a byte stream. The TCP transport uses length-prefixed framing:

```
[4 bytes: frame length (big-endian u32)] [frame payload]
```

This is handled automatically by `TcpTransport` -- you send and receive complete CLASP frames without worrying about boundaries.

## Performance

| Metric | Typical Value |
|--------|---------------|
| Connection setup | 1-5ms (LAN) |
| Message latency | 0.5-2ms (LAN) |
| Throughput | 100,000+ msg/sec |
| Overhead per message | 4 bytes (length prefix) |

TCP has slightly lower overhead than WebSocket (no WebSocket frame header), but the difference is negligible for most use cases.

## Comparison

| Aspect | TCP | WebSocket |
|--------|-----|-----------|
| Framing | Length-prefixed | WebSocket frames |
| HTTP compatible | No | Yes (upgrade) |
| Proxy traversal | No | Yes |
| Browser support | No | Yes |
| Relay support | Embed only | Direct |
| Overhead | Minimal | Minimal (+2-14 bytes) |

## Troubleshooting

**Connection refused** -- Make sure your server binary is listening on the expected port. Remember, `clasp-relay` does not accept TCP -- you need to embed `clasp-router`.

**Connection drops on WAN** -- Raw TCP connections are often blocked by firewalls and NATs. Use WebSocket for anything outside a trusted LAN.

## See Also

- [WebSocket Transport](websocket.md) -- the default, works through proxies
- [UDP Transport](udp.md) -- unreliable but lowest latency
- [Core Concepts: Transports](../core/transports.md) -- how transports fit into CLASP
