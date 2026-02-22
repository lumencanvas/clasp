---
title: "UDP Transport"
description: "Lowest latency transport for CLASP."
section: reference
order: 4
---
# UDP Transport

Lowest latency transport for CLASP.

## Overview

UDP transport provides the lowest latency for CLASP communication, ideal for LAN environments where occasional message loss is acceptable.

## Connection URLs

```
udp://host:port
```

## Features

| Feature | Support |
|---------|---------|
| Bidirectional | Yes |
| Reliable delivery | No |
| Ordered delivery | No |
| Browser support | No |
| TLS encryption | No* |
| Proxy compatible | No |
| Connection overhead | None |
| Latency | Lowest |

*Use DTLS for encryption

## Use Cases

- High-frequency sensor data
- Real-time control (lighting, audio)
- LAN-only applications
- Embedded systems

## Server Configuration

```yaml
server:
  udp:
    enabled: true
    port: 7331
    bind: "0.0.0.0"
```

### CLI

```bash
clasp server --udp --udp-port 7331
```

## Client Usage

### Rust

```rust
use clasp_transport::udp::UdpTransport;

let transport = UdpTransport::new(UdpConfig {
    bind_addr: "0.0.0.0:0".parse()?,
    target_addr: "192.168.1.100:7331".parse()?,
}).await?;
```

### Embedded (no_std)

```rust
// Prepare frame for UDP transmission
let frame = client.prepare_set("/sensor/value", Value::Float(temp));
udp_socket.send(&frame);
```

## Message Format

UDP datagrams contain complete CLASP frames:

```
┌─────────────────────────────────────────┐
│ UDP Datagram                            │
├─────────────────────────────────────────┤
│ CLASP Frame (same as WebSocket)         │
└─────────────────────────────────────────┘
```

Maximum message size: 65,507 bytes (practical: ~1400 for MTU)

## Reliability

UDP provides no delivery guarantees. For critical messages:

```javascript
// Use CLASP QoS for important messages
client.setWithQos('/critical/value', value, 'confirm');
```

The CLASP protocol handles retransmission at the application layer when using Confirm or Commit QoS.

## Multicast

For broadcasting to multiple receivers:

```yaml
server:
  udp:
    multicast:
      enabled: true
      groups:
        - "239.255.0.1"
```

```rust
// Client subscribes to multicast group
let transport = UdpTransport::new(UdpConfig {
    bind_addr: "0.0.0.0:7331".parse()?,
    multicast_groups: vec!["239.255.0.1".parse()?],
}).await?;
```

## Broadcast

For LAN discovery and simple broadcast:

```rust
let transport = UdpTransport::new(UdpConfig {
    bind_addr: "0.0.0.0:7331".parse()?,
    broadcast_enabled: true,
}).await?;

transport.send_to(data, "255.255.255.255:7331").await?;
```

## Performance

### Typical Metrics

- Latency: ~0.1-1ms (LAN)
- Throughput: 200,000+ msg/sec
- Overhead: 8 bytes (UDP header)

### Optimization

```yaml
server:
  udp:
    recv_buffer: 1048576  # 1MB
    send_buffer: 1048576
```

## Comparison

| Aspect | UDP | WebSocket | QUIC |
|--------|-----|-----------|------|
| Latency | Lowest | Low | Very Low |
| Reliability | None | Full | Full |
| Ordering | None | Yes | Per-stream |
| Setup time | None | ~20ms | ~10ms |
| Browser | No | Yes | Limited |

## When to Use UDP

**Use UDP when:**
- Sub-millisecond latency required
- LAN-only communication
- High message rates (>10,000/sec)
- Occasional loss acceptable
- Embedded systems with limited resources

**Don't use UDP when:**
- Messages must be delivered
- Ordering matters
- Going over internet
- Browser clients needed

## Error Handling

UDP doesn't report delivery failures. Implement application-level acknowledgment:

```javascript
// Send with sequence number
await client.set('/sensor/data', {
  seq: seqNum++,
  value: sensorValue
});

// Receiver detects gaps in sequence
```

## See Also

- [WebSocket Transport](websocket.md)
- [QUIC Transport](quic.md)
- [Embedded Systems](../../use-cases/embedded-systems.md)
