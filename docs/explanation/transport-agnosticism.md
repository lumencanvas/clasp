# Transport Agnosticism

CLASP is designed to work over any byte transport. This document explains why and how.

## The Design Philosophy

CLASP separates **protocol** from **transport**:

```
┌────────────────────────────────┐
│       CLASP Protocol           │  ← Messages, types, semantics
├────────────────────────────────┤
│       Transport Layer          │  ← How bytes move
│  WebSocket │ QUIC │ UDP │ ... │
└────────────────────────────────┘
```

The same CLASP messages work over any transport.

## Why Transport Agnostic?

### Different Use Cases Need Different Transports

| Use Case | Best Transport | Why |
|----------|---------------|-----|
| Browser app | WebSocket | Only option in browsers |
| Native app | QUIC | Connection migration, multiplexing |
| LAN device | UDP | Lowest latency, minimal overhead |
| Embedded | HTTP POST | Simple, works through firewalls |
| P2P | WebRTC | NAT traversal |
| Hardware | Serial | Direct connection |
| Wireless IoT | BLE | Low power |

One transport can't serve all needs.

### Protocol Evolution

Transports evolve faster than protocols:
- QUIC didn't exist when OSC was designed
- WebRTC emerged after MIDI
- New transports will emerge

A transport-agnostic protocol adapts to new transports.

## Supported Transports

### WebSocket

**Best for:** Browser apps, cross-platform baseline

```javascript
const client = new Clasp('ws://localhost:7330');
// or with TLS
const client = new Clasp('wss://server.com:7330');
```

Characteristics:
- Stream-based (reliable, ordered)
- Works in browsers
- Widely supported
- ~1-5ms latency on LAN

### QUIC

**Best for:** Native apps, mobile, unreliable networks

```rust
let client = ClaspBuilder::new("quic://server.com:7331")
    .connect()
    .await?;
```

Characteristics:
- UDP-based with reliability
- TLS 1.3 mandatory
- 0-RTT connection resumption
- Connection migration (IP changes)
- Multiplexed streams

### UDP

**Best for:** LAN devices, lowest latency

```rust
let client = ClaspBuilder::new("udp://192.168.1.100:7331")
    .connect()
    .await?;
```

Characteristics:
- No reliability (use for streams)
- No connection setup
- Broadcast capable
- Lowest overhead
- LAN only (NAT issues)

### WebRTC DataChannel

**Best for:** P2P, browser-to-browser

```javascript
const dc = peerConnection.createDataChannel('clasp', {
  ordered: false,
  maxRetransmits: 0  // For Q0 streams
});
```

Characteristics:
- P2P (no server required)
- NAT traversal via ICE/STUN/TURN
- Configurable reliability
- Browser support

### Serial

**Best for:** Hardware, embedded devices

```rust
let client = ClaspBuilder::new("serial:///dev/ttyUSB0?baud=115200")
    .connect()
    .await?;
```

Characteristics:
- Direct hardware connection
- No network stack
- Lowest latency possible
- Point-to-point only

### Bluetooth Low Energy (BLE)

**Best for:** Wireless IoT, battery devices

Characteristics:
- Low power
- Short range (~10m)
- Limited bandwidth
- Mobile device support

### HTTP POST

**Best for:** Embedded, firewall traversal

```bash
curl -X POST http://server:3000/api/set \
  -d '{"address":"/sensor/temp","value":23.5}'
```

Characteristics:
- Works through any firewall
- Stateless (no subscriptions)
- Higher latency
- Simple to implement

## Choosing a Transport

### Decision Matrix

| Requirement | Transport |
|-------------|-----------|
| Browser support | WebSocket |
| Mobile app | QUIC or WebSocket |
| Lowest latency (LAN) | UDP |
| Lowest latency (WAN) | QUIC |
| P2P | WebRTC |
| Firewall traversal | WebSocket, HTTP, QUIC |
| Embedded (simple) | HTTP POST, UDP |
| Hardware control | Serial |
| Wireless IoT | BLE |

### Mixed Transport Systems

A single CLASP system can use multiple transports:

```
Browser (WebSocket) ─────┐
                         │
Native App (QUIC) ───────┼──► Router (accepts all)
                         │
ESP32 (HTTP POST) ───────┘
```

The router accepts connections from any supported transport.

## Transport-Specific Considerations

### QoS Mapping

| CLASP QoS | WebSocket | UDP | WebRTC |
|-----------|-----------|-----|--------|
| Q0 Fire | Send | Send | Unreliable channel |
| Q1 Confirm | Send + wait ACK | App-level retry | Reliable channel |
| Q2 Commit | Send + wait ACK + persist | N/A | Reliable channel |

### Framing

Each transport handles CLASP frames differently:

| Transport | Framing |
|-----------|---------|
| WebSocket | One frame per WS message |
| UDP | One frame per datagram |
| QUIC | One frame per stream message |
| Serial | Length-prefixed, magic byte for sync |

### MTU Handling

```
WebSocket: No limit (transport handles fragmentation)
UDP: ~1400 bytes (MTU limit)
BLE: ~512 bytes (characteristic limit)
```

For large messages over limited transports:
- Split at application level
- Use streaming
- Compress

## Implementing New Transports

CLASP's transport abstraction makes adding new transports straightforward:

```rust
trait Transport {
    async fn connect(url: &str) -> Result<Self>;
    async fn send(&self, frame: &[u8]) -> Result<()>;
    async fn recv(&self) -> Result<Vec<u8>>;
    async fn close(&self) -> Result<()>;
}
```

Any transport that can send/receive bytes can carry CLASP.

## Interoperability

The recommended approach for maximum interoperability:

1. **Implement WebSocket** (baseline, browsers can connect)
2. **Add QUIC** (better performance for native)
3. **Add UDP** (LAN performance)
4. **Add others** (as needed)

Routers should support WebSocket at minimum to ensure any client can connect.

## See Also

- [WebSocket Transport](../reference/transports/websocket.md)
- [QUIC Transport](../reference/transports/quic.md)
- [UDP Transport](../reference/transports/udp.md)
- [WebRTC Transport](../reference/transports/webrtc.md)
