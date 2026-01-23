# WebRTC Transport

Peer-to-peer transport for CLASP.

## Overview

WebRTC DataChannels enable direct peer-to-peer communication between clients, bypassing the central router for lower latency and reduced server load.

## Features

| Feature | Support |
|---------|---------|
| Bidirectional | Yes |
| Reliable delivery | Configurable |
| Ordered delivery | Configurable |
| Browser support | Yes |
| TLS encryption | Yes (DTLS) |
| NAT traversal | Yes |
| Connection overhead | High (setup) |
| Latency | Low (after setup) |

## Architecture

```
Standard:   Client A → Router → Client B

WebRTC:     Client A ←────────→ Client B
                    (DataChannel)
```

## Setup Requirements

### Signaling Server

Exchange connection information through CLASP:

```javascript
// Using CLASP router for signaling
client.on('/webrtc/signal/**', async (signal, address) => {
  const [, , , from, to] = address.split('/');
  await client.emit(`/webrtc/signal/${to}/${from}`, signal);
});
```

### ICE Servers

STUN/TURN for NAT traversal:

```javascript
const rtcConfig = {
  iceServers: [
    { urls: 'stun:stun.l.google.com:19302' },
    {
      urls: 'turn:turn.example.com:3478',
      username: 'user',
      credential: 'pass'
    }
  ]
};
```

## Client Usage

### Browser

```javascript
import { ClaspP2P } from '@clasp-to/webrtc';

const p2p = new ClaspP2P({
  signaling: 'ws://localhost:7330',
  rtcConfig: rtcConfig,
  clientId: 'browser-1'
});

await p2p.connect();

// Connect to peer
const peer = await p2p.connectToPeer('browser-2');

// Send/receive
peer.set('/local/value', 42);
peer.on('/local/**', (value, address) => {
  console.log(address, value);
});
```

### Node.js

```javascript
const { ClaspP2P } = require('@clasp-to/webrtc');
const wrtc = require('wrtc');  // WebRTC for Node

const p2p = new ClaspP2P({
  signaling: 'ws://localhost:7330',
  rtcConfig: rtcConfig,
  clientId: 'node-1',
  wrtc: wrtc
});
```

## DataChannel Configuration

### Reliable (Default)

Ordered, guaranteed delivery:

```javascript
const channel = peer.createDataChannel('/control', {
  ordered: true,
  maxRetransmits: null  // Unlimited
});
```

### Unreliable

Unordered, best-effort (like UDP):

```javascript
const channel = peer.createDataChannel('/stream', {
  ordered: false,
  maxRetransmits: 0
});
```

### Partially Reliable

Limited retransmissions:

```javascript
const channel = peer.createDataChannel('/media', {
  ordered: true,
  maxRetransmits: 3,
  maxPacketLifeTime: 100  // ms
});
```

## Connection Lifecycle

```
1. Client A creates offer
2. Client A sends offer via signaling (CLASP)
3. Client B receives offer, creates answer
4. Client B sends answer via signaling
5. ICE candidates exchanged via signaling
6. Direct connection established
7. DataChannel communication
```

## NAT Traversal

### STUN

For most home networks:

```javascript
{ urls: 'stun:stun.l.google.com:19302' }
```

### TURN

For symmetric NATs (fallback):

```javascript
{
  urls: 'turn:turn.example.com:3478',
  username: 'user',
  credential: 'pass'
}
```

### ICE Candidates

Connection attempts through different paths:

```
1. Host candidate (direct LAN)
2. Server reflexive (STUN)
3. Relay candidate (TURN)
```

## Hybrid Mode

Combine P2P with router:

```javascript
const client = await Clasp.builder('ws://localhost:7330')
  .withP2P({
    enabled: true,
    rtcConfig: rtcConfig,
    preferP2P: true,
    fallbackToRouter: true
  })
  .connect();

// Automatically uses P2P when available
await client.set('/peer/browser-2/value', 42);
```

## Performance

### After Connection

- Latency: ~1-10ms (LAN), ~10-50ms (WAN)
- Throughput: 10,000+ msg/sec
- Encryption: DTLS (always on)

### Connection Setup

- Time: 100ms - 5s (varies by NAT)
- Signaling: 2-6 round trips

## Security

- **DTLS encryption**: Always enabled
- **Certificate fingerprints**: Verified during handshake
- **Identity**: Validated through signaling

## Troubleshooting

### Connection Fails

1. Check signaling server works
2. Add more STUN servers
3. Add TURN server for difficult NATs
4. Check firewall allows UDP

### High Latency

1. Verify P2P connection (not TURN relay)
2. Check ICE connection state
3. Prefer host candidates

## See Also

- [P2P WebRTC How-To](../../how-to/advanced/p2p-webrtc.md)
- [WebSocket Transport](websocket.md)
