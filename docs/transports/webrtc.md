---
title: WebRTC
description: Peer-to-peer transport for direct browser-to-browser communication
order: 8
---

# WebRTC Transport

WebRTC DataChannels enable direct peer-to-peer communication between CLASP clients, bypassing the relay entirely. Two browsers (or native clients) talk directly to each other with DTLS encryption.

## When to Use WebRTC

- **Browser-to-browser** without routing through a server
- **Lowest possible latency** between two specific peers
- **Reduced server load** -- the relay doesn't see the traffic
- **Privacy** -- data never touches your infrastructure

WebRTC still needs a signaling mechanism to set up the connection (exchanging offers/answers/ICE candidates). CLASP uses the relay itself for signaling, then the peers communicate directly.

## How It Works

```
NORMAL (through relay):
  Client A ──> clasp-relay ──> Client B

WEBRTC (direct):
  Client A ←─────────────────→ Client B
           (WebRTC DataChannel)

  The relay is only used briefly for signaling (connection setup).
  Once the DataChannel is open, all traffic is peer-to-peer.
```

### Connection Setup

1. Client A creates a WebRTC offer
2. Client A sends the offer to Client B via the relay (signaling)
3. Client B receives the offer, creates an answer
4. Client B sends the answer back via the relay
5. Both sides exchange ICE candidates via the relay
6. Direct DataChannel connection is established
7. All subsequent CLASP frames flow peer-to-peer

## Feature Flag

WebRTC is not included in default features:

```bash
cargo build --features webrtc
```

## Rust Transport API

```rust
use clasp_transport::webrtc::{WebRtcTransport, WebRtcConfig};

let config = WebRtcConfig {
    ice_servers: vec![
        "stun:stun.l.google.com:19302".into(),
    ],
    ..Default::default()
};

let (sender, receiver) = WebRtcTransport::connect(config, signaling_channel).await?;

// Same trait as every other transport
sender.send(clasp_frame).await?;
```

## Browser Usage

```javascript
import { ClaspP2P } from '@clasp-to/webrtc'

const p2p = new ClaspP2P({
  signaling: 'ws://localhost:7330',
  rtcConfig: {
    iceServers: [
      { urls: 'stun:stun.l.google.com:19302' }
    ]
  },
  clientId: 'browser-1'
})

await p2p.connect()

// Connect to a specific peer
const peer = await p2p.connectToPeer('browser-2')
peer.set('/local/value', 42)
peer.on('/local/**', (value, address) => {
  console.log(address, value)
})
```

## ICE Servers (NAT Traversal)

Most devices are behind NATs. ICE servers help peers find a path to each other:

**STUN** -- tells each peer its public IP. Works for most home networks:
```javascript
{ urls: 'stun:stun.l.google.com:19302' }
```

**TURN** -- relays traffic when direct connection isn't possible (symmetric NATs, strict firewalls). This is a fallback -- traffic goes through the TURN server, not peer-to-peer:
```javascript
{
  urls: 'turn:turn.example.com:3478',
  username: 'user',
  credential: 'pass'
}
```

ICE tries paths in order: direct LAN > STUN (public IP) > TURN (relay fallback).

## DataChannel Reliability Modes

WebRTC DataChannels can be configured per-channel:

### Reliable (default)
Ordered, guaranteed delivery. Behaves like TCP:
```javascript
const channel = peer.createDataChannel('/control', {
  ordered: true
})
```

### Unreliable
Unordered, best-effort. Behaves like UDP -- great for real-time data:
```javascript
const channel = peer.createDataChannel('/stream', {
  ordered: false,
  maxRetransmits: 0
})
```

### Partially Reliable
Limited retransmissions -- a middle ground:
```javascript
const channel = peer.createDataChannel('/media', {
  ordered: true,
  maxRetransmits: 3
})
```

## Hybrid Mode

Use both the relay and P2P connections. The relay provides shared state and signaling; P2P provides low-latency direct communication between specific peers:

```javascript
const client = await new ClaspBuilder('ws://localhost:7330')
  .withP2P({
    enabled: true,
    rtcConfig: { iceServers: [...] },
    preferP2P: true,
    fallbackToRouter: true
  })
  .connect()

// Automatically uses P2P when available, falls back to relay
await client.set('/peer/browser-2/value', 42)
```

## Performance

| Metric | Typical Value |
|--------|---------------|
| Connection setup | 100ms - 5s (varies by NAT) |
| Message latency | 1-10ms (LAN), 10-50ms (WAN) |
| Throughput | 10,000+ msg/sec |
| Encryption | DTLS (always on) |

Connection setup is slow compared to other transports because of the ICE negotiation process. Once connected, latency is excellent.

## Troubleshooting

**Connection fails** -- Add more STUN servers. If behind a strict firewall or symmetric NAT, you need a TURN server as fallback. Check that UDP is allowed.

**High latency** -- Verify you have a direct connection, not a TURN relay. Check ICE connection state -- "relay" means traffic is going through TURN.

**Works on LAN but not over internet** -- You likely need a TURN server. STUN alone doesn't work with all NAT types.

## See Also

- [WebSocket Transport](websocket.md) -- simpler setup, goes through relay
- [Core Concepts: Transports](../core/transports.md) -- how transports fit into CLASP
- [P2P & WebRTC](../core/p2p.md) -- conceptual overview of P2P in CLASP
