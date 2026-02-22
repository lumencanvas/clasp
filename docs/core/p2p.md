---
title: P2P & WebRTC
description: Direct peer-to-peer communication without routing through the relay
order: 6
---

# P2P & WebRTC

CLASP supports direct peer-to-peer communication using WebRTC DataChannels. P2P connections bypass the relay entirely, giving you lower latency and higher throughput for use cases like video streaming, large file transfers, and privacy-sensitive data.

## When to Use P2P vs Relay

| Factor | Relay | P2P |
|--------|-------|-----|
| Latency | Server round-trip | Direct (sub-millisecond on LAN) |
| Throughput | Limited by server bandwidth | Limited only by peer connection |
| Privacy | Data passes through server | Data stays between peers |
| NAT traversal | Server handles it | Requires STUN/TURN |
| Reliability | Server guarantees delivery | Best-effort (unreliable channel) or ordered (reliable channel) |
| Discovery | Built-in addressing | Requires rendezvous server |
| Scale | Hundreds of clients | Best for 2-10 peers |

**Use P2P for**: video/audio streaming, large sensor payloads, latency-critical control, private data exchange.

**Use the relay for**: many-to-many broadcast, state persistence, late-joiner sync, NAT-unfriendly networks.

## P2PManager

The `P2PManager` class handles peer discovery, signaling, and WebRTC connection setup. It uses a rendezvous server (typically the CLASP relay) for initial peer discovery and SDP/ICE exchange, then establishes direct DataChannel connections.

### JavaScript

```javascript
import { P2PManager } from '@clasp-to/core';

const p2p = new P2PManager({
  peerId: 'my-peer',
  rendezvousUrl: 'https://relay.clasp.to',
  iceServers: [
    { urls: 'stun:stun.l.google.com:19302' },
    { urls: 'stun:stun1.l.google.com:19302' }
  ],
  useUnreliableChannel: true
});
```

### Python

```python
from clasp_to import P2PManager

p2p = P2PManager(
    peer_id='my-peer',
    rendezvous_url='https://relay.clasp.to',
    ice_servers=[
        {'urls': 'stun:stun.l.google.com:19302'},
        {'urls': 'stun:stun1.l.google.com:19302'},
    ],
    use_unreliable_channel=True,
)
```

### Rust

```rust
use clasp_transport::p2p::{P2PConfig, P2PManager};

let config = P2PConfig {
    peer_id: "my-peer".to_string(),
    rendezvous_url: "https://relay.clasp.to".to_string(),
    ice_servers: vec![
        "stun:stun.l.google.com:19302".to_string(),
        "stun:stun1.l.google.com:19302".to_string(),
    ],
    use_unreliable_channel: true,
    ..Default::default()
};

let p2p = P2PManager::new(config).await?;
```

### Constructor Options

| Option | Type | Description |
|--------|------|-------------|
| `peerId` | `string` | Unique identifier for this peer |
| `rendezvousUrl` | `string` | URL of the rendezvous/signaling server |
| `iceServers` | `array` | STUN/TURN server configuration |
| `useUnreliableChannel` | `bool` | Enable the unreliable (unordered) DataChannel for streams |

## Registration

Before other peers can discover you, register with the rendezvous server:

```javascript
await p2p.register({
  tags: ['demo', 'webrtc'],
  metadata: {
    name: 'My Peer',
    capabilities: ['chat', 'sensors']
  }
});
```

Tags allow filtering during discovery. Metadata is freeform and returned to discovering peers.

## Discovery

Find other registered peers, optionally filtering by tags:

```javascript
const peers = await p2p.discover({ tags: ['demo'] });
for (const p of peers) {
  console.log(`${p.id}: ${p.metadata?.name}`);
}
```

In Rust, `discover` takes an optional tag list and a limit:

```rust
let peers = p2p.discover(Some(vec!["demo".to_string()]), 100).await?;
```

## Connecting

Initiate a direct connection to a discovered peer:

```javascript
const peer = await p2p.connect('peer-b');
console.log(`Connected to ${peer.id}`);
```

The `connect()` call handles SDP offer/answer exchange and ICE candidate negotiation behind the scenes. It resolves once the DataChannel is open.

## Peer Operations

Once connected, a peer object supports CLASP-style operations over the direct connection:

```javascript
// Set a value (reliable channel -- ordered delivery)
peer.set('/chat/greeting', 'Hello!');

// Stream data (unreliable channel -- low latency, no ordering)
peer.stream('/sensor/accel', { x: 0.1, y: -0.3, z: 9.8 });

// Subscribe to incoming data
peer.on('/chat/*', (value, address) => {
  console.log(`${address}: ${value}`);
});

// Emit a one-shot event
peer.emit('/action/ping', { ts: Date.now() });
```

### Dual Channels

When `useUnreliableChannel` is enabled, the P2PManager creates two DataChannels:

| Channel | Ordering | Use Case |
|---------|----------|----------|
| **Reliable** | Ordered, guaranteed delivery | State (`set`), events (`emit`), control messages |
| **Unreliable** | Unordered, best-effort | Streams (`stream`), video frames, sensor data |

`set()` and `emit()` always use the reliable channel. `stream()` uses the unreliable channel when available, falling back to reliable if not.

## Events

Handle connection lifecycle events:

```javascript
p2p.on('connection', (peer) => {
  console.log(`Peer connected: ${peer.id}`);
});

p2p.on('disconnection', (peerId) => {
  console.log(`Peer disconnected: ${peerId}`);
});

p2p.on('error', (err) => {
  console.error(`Error: ${err.message}`);
});
```

In Rust, use the `connections()` receiver:

```rust
let mut rx = p2p.connections();
while let Some(peer) = rx.recv().await {
    println!("Peer connected: {}", peer.id());
}
```

## ICE/STUN/TURN Configuration

WebRTC requires ICE servers for NAT traversal. STUN servers help peers discover their public IP. TURN servers relay traffic when direct connections fail.

```javascript
const p2p = new P2PManager({
  peerId: 'my-peer',
  rendezvousUrl: 'https://relay.clasp.to',
  iceServers: [
    // Public STUN (free, no auth)
    { urls: 'stun:stun.l.google.com:19302' },

    // Private TURN (for NAT-heavy networks)
    {
      urls: 'turn:turn.example.com:3478',
      username: 'user',
      credential: 'pass'
    }
  ]
});
```

For production, you should run your own TURN server (e.g., coturn) to guarantee connectivity when peers are behind symmetric NATs.

## Auto-Fallback to Relay

If a P2P connection fails (ICE negotiation timeout, incompatible NATs, firewall restrictions), the P2PManager can automatically fall back to routing through the CLASP relay. This is transparent to application code -- `peer.set()` and `peer.stream()` continue to work, just with higher latency.

## Complete Example (JavaScript)

```javascript
import { P2PManager } from '@clasp-to/core';

const PEER_ID = process.env.PEER_ID || `peer-${Date.now()}`;
const CONNECT_TO = process.env.CONNECT_TO;

const p2p = new P2PManager({
  peerId: PEER_ID,
  rendezvousUrl: 'https://relay.clasp.to',
  iceServers: [
    { urls: 'stun:stun.l.google.com:19302' },
    { urls: 'stun:stun1.l.google.com:19302' }
  ],
  useUnreliableChannel: true
});

// Handle incoming connections
p2p.on('connection', (peer) => {
  console.log(`Peer connected: ${peer.id}`);

  peer.on('/chat/*', (value, address) => {
    console.log(`[${peer.id}] ${address}: ${value}`);
  });

  peer.on('/sensor/accel', (value) => {
    console.log(`[${peer.id}] Accel: x=${value.x}, y=${value.y}, z=${value.z}`);
  });
});

p2p.on('disconnection', (peerId) => {
  console.log(`Peer disconnected: ${peerId}`);
});

// Register
await p2p.register({
  tags: ['demo', 'webrtc'],
  metadata: { name: `Demo Peer ${PEER_ID}` }
});

// Connect to a specific peer (if specified)
if (CONNECT_TO) {
  const peer = await p2p.connect(CONNECT_TO);

  // Reliable channel: state
  peer.set('/chat/greeting', `Hello from ${PEER_ID}!`);

  // Unreliable channel: high-rate sensor stream
  setInterval(() => {
    const t = Date.now() / 1000;
    peer.stream('/sensor/accel', {
      x: Math.sin(t),
      y: Math.cos(t),
      z: Math.sin(t * 0.5)
    });
  }, 100);
}

// Discover other peers
const peers = await p2p.discover({ tags: ['demo'] });
console.log(`Found ${peers.length} peer(s)`);
```

Run in two terminals:

```bash
# Terminal 1
PEER_ID=peer-a node p2p-webrtc.js

# Terminal 2
PEER_ID=peer-b CONNECT_TO=peer-a node p2p-webrtc.js
```

## Video Streaming

P2P is the primary transport for video in CLASP. There are two approaches depending on your topology.

### P2P Video (Direct WebRTC)

For 1:1 or small-group video, use the P2PManager's unreliable channel to stream video frames directly between peers. The `examples/js/video-p2p.html` example uses CLASP for signaling (SDP/ICE exchange) and native WebRTC `RTCPeerConnection` for the media stream:

1. Peers join a room by setting presence at `/video/room/{room}/presence/{session}`
2. When a new peer appears, the lexicographically-higher session creates an SDP offer
3. ICE candidates and SDP answers are exchanged via CLASP events at `/video/room/{room}/signal/{peerId}`
4. Once connected, video flows directly over WebRTC -- not through the CLASP relay

```javascript
// Simplified flow (see examples/js/video-p2p.html for full code)
const builder = new ClaspBuilder(url);
builder.name('video-p2p');
builder.reconnect(true);
const client = await builder.connect();

// Announce presence
client.set(`/video/room/${room}/presence/${client.session}`, {
  name: 'my-peer',
  joinedAt: Date.now()
});

// Subscribe to signaling
client.on(`/video/room/${room}/signal/${client.session}`, (data) => {
  // Handle SDP offers, answers, and ICE candidates
  handleSignal(data);
});
```

### Relay Video (Stream Signal)

For broadcast scenarios (one sender, many viewers), use CLASP Stream signals through the relay. The sender encodes video frames with WebCodecs, chunks them, and sends each chunk as a Stream message. Receivers reassemble chunks and decode:

```javascript
// Sender: encode and chunk video frames
const address = `/video/relay/${room}/stream/${client.session}`;

function handleEncodedFrame(chunk, metadata) {
  const data = new Uint8Array(chunk.byteLength);
  chunk.copyTo(data);

  // Split into 16KB chunks for CLASP transport
  const CHUNK_SIZE = 16000;
  const totalChunks = Math.ceil(data.byteLength / CHUNK_SIZE);

  for (let i = 0; i < totalChunks; i++) {
    client.stream(address, {
      seq: frameSeq,
      chunkIndex: i,
      totalChunks,
      frameType: chunk.type,
      timestamp: chunk.timestamp,
      data: data.slice(i * CHUNK_SIZE, (i + 1) * CHUNK_SIZE)
    });
  }
}
```

### Choosing an Approach

| Factor | P2P Video | Relay Video |
|--------|-----------|-------------|
| Topology | 1:1 or small group | 1:many broadcast |
| Latency | Lowest (direct) | Higher (server hop) |
| Codec | WebRTC native (VP8/VP9/H.264) | WebCodecs (manual encode/decode) |
| NAT traversal | Requires STUN/TURN | Relay handles it |
| Bandwidth | Peer uploads to each viewer | Single upload, server fans out |

## Next Steps

- [Signal Types](./signals.md) -- understand when to use streams vs params vs events
- [Clock Sync & Timing](./timing.md) -- synchronize actions across peers
- [JavaScript SDK](../sdk/javascript.md) -- full client API reference
