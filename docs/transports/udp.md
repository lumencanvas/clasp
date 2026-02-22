---
title: UDP
description: Lowest-latency transport for fire-and-forget sensor data and real-time control
order: 5
---

# UDP Transport

UDP provides the absolute lowest latency of any CLASP transport. There's no connection setup, no retransmission, no ordering -- just raw datagrams. If a packet is lost, it's gone.

## When to Use UDP

- High-frequency sensor data where the latest reading is all that matters
- Real-time control (lighting, audio) where a stale retransmit is worse than a skip
- LAN-only communication where packet loss is rare
- Embedded systems with minimal resources
- Discovery and broadcast

## When NOT to Use UDP

- Messages that must be delivered (use WebSocket or QUIC)
- Communication over the internet (packet loss is common)
- Anything that needs ordering guarantees
- Browser clients (browsers can't send raw UDP)

## Important: The Relay Does Not Listen on UDP

Like TCP, the relay binary only speaks WebSocket + QUIC. To accept UDP, embed `clasp-router` in your own binary:

```rust
use clasp_router::Router;
use clasp_transport::udp::{UdpTransport, UdpConfig};

let router = Router::new();
let config = UdpConfig {
    bind_addr: "0.0.0.0:7341".parse()?,
    ..Default::default()
};
let (sender, receiver) = UdpTransport::bind(config).await?;
router.add_transport(sender, receiver).await;
```

## Rust Transport API

### Point-to-Point

```rust
use clasp_transport::udp::{UdpTransport, UdpConfig};

let config = UdpConfig {
    bind_addr: "0.0.0.0:0".parse()?,        // any local port
    target_addr: "192.168.1.100:7341".parse()?, // destination
    ..Default::default()
};

let (sender, receiver) = UdpTransport::new(config).await?;

sender.send(clasp_frame).await?;
```

### Broadcast

```rust
let config = UdpConfig {
    bind_addr: "0.0.0.0:7341".parse()?,
    broadcast_enabled: true,
    ..Default::default()
};

let broadcast = UdpBroadcast::new(config).await?;
broadcast.send_to(data, "255.255.255.255:7341").await?;
```

### Multicast

```rust
let config = UdpConfig {
    bind_addr: "0.0.0.0:7341".parse()?,
    multicast_groups: vec!["239.255.0.1".parse()?],
    ..Default::default()
};

let (sender, receiver) = UdpTransport::new(config).await?;
```

## Message Format

Each UDP datagram contains exactly one CLASP binary frame. No additional framing is needed -- UDP datagrams have natural message boundaries.

```
┌─────────────────────────────┐
│ UDP Datagram                │
│ ┌─────────────────────────┐ │
│ │ CLASP Binary Frame      │ │
│ └─────────────────────────┘ │
└─────────────────────────────┘
```

Maximum practical payload: ~1400 bytes (to stay within typical MTU). The hard UDP limit is 65,507 bytes, but fragmented datagrams are unreliable.

## Reliability: There Is None

UDP does not guarantee delivery, ordering, or deduplication. If you need reliability for specific messages, use CLASP's application-level QoS:

```rust
// Confirm QoS adds application-level acknowledgment
client.set_with_qos("/critical/value", value, QoS::Confirm).await?;
```

For most UDP use cases (sensors, real-time control), you don't want reliability -- the next reading replaces the last one anyway.

## Performance

| Metric | Typical Value |
|--------|---------------|
| Connection setup | None (connectionless) |
| Message latency | 0.1-1ms (LAN) |
| Throughput | 200,000+ msg/sec |
| Overhead | 8 bytes (UDP header only) |

## Comparison

| Aspect | UDP | TCP | WebSocket |
|--------|-----|-----|-----------|
| Latency | Lowest | Low | Low |
| Reliability | None | Full | Full |
| Ordering | None | Yes | Yes |
| Setup time | None | ~5ms | ~20ms |
| Browser support | No | No | Yes |
| Overhead | 8 bytes | 20 bytes | 22-34 bytes |

## Troubleshooting

**No data received** -- UDP is silent when things fail. Verify the destination address/port, check firewall rules, and confirm both sides are on the same subnet for broadcast.

**Missing messages** -- This is expected with UDP. If you're seeing high loss rates on a LAN, check for network congestion, buffer overflows, or MTU issues.

**Large messages not arriving** -- Keep payloads under ~1400 bytes to avoid IP fragmentation. Fragmented UDP datagrams are much more likely to be lost.

## See Also

- [TCP Transport](tcp.md) -- reliable alternative for LAN use
- [WebSocket Transport](websocket.md) -- reliable and works everywhere
- [Core Concepts: Transports](../core/transports.md) -- how transports fit into CLASP
