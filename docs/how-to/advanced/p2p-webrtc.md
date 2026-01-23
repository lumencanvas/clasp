# P2P WebRTC

Connect CLASP clients directly using WebRTC DataChannels for peer-to-peer communication.

## Overview

WebRTC enables direct connections between browsers and other clients without routing through a server. This is useful for:

- Low-latency local communication
- Reduced server load
- Working when central router is unavailable

## Architecture

```
Normal:     Client A → Router → Client B

P2P:        Client A ←──────→ Client B
                    (WebRTC)
```

## Setup

### Signaling Server

WebRTC requires a signaling server to exchange connection information:

```javascript
// Simple signaling server using CLASP router
client.on('/webrtc/signal/**', async (signal, address) => {
  // Forward signaling messages between peers
  const [, , , from, to] = address.split('/');
  await client.emit(`/webrtc/signal/${to}/${from}`, signal);
});
```

### STUN/TURN Configuration

For connections across NATs:

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

## Browser Client

### Initialize P2P

```javascript
import { ClaspP2P } from '@clasp-to/webrtc';

const p2p = new ClaspP2P({
  signaling: 'ws://localhost:7330',  // Signaling through CLASP
  rtcConfig: rtcConfig,
  clientId: 'browser-1'
});

await p2p.connect();
```

### Connect to Peer

```javascript
// Initiate connection to another peer
const peer = await p2p.connectToPeer('browser-2');

// Now communicate directly
peer.set('/local/value', 42);

peer.on('/local/**', (value, address) => {
  console.log(`Received from peer: ${address} = ${value}`);
});
```

### Accept Incoming Connections

```javascript
p2p.on('peer_connected', (peer) => {
  console.log(`Peer ${peer.id} connected`);

  peer.on('/control/**', (value, address) => {
    handleControl(address, value);
  });
});
```

## Hybrid Mode

Use P2P for local, router for remote:

```javascript
const client = await Clasp.builder('ws://localhost:7330')
  .withP2P({
    enabled: true,
    rtcConfig: rtcConfig,
    preferP2P: true,  // Use P2P when available
    fallbackToRouter: true
  })
  .connect();

// Automatically uses P2P if peer is local
await client.set('/peer/browser-2/value', 42);
```

## Data Channels

### Reliable (Default)

```javascript
// Ordered, reliable delivery (like TCP)
const channel = peer.createDataChannel('/control', {
  ordered: true,
  maxRetransmits: null  // Unlimited retries
});
```

### Unreliable (Low Latency)

```javascript
// Unordered, best-effort (like UDP)
const channel = peer.createDataChannel('/stream', {
  ordered: false,
  maxRetransmits: 0  // No retries
});
```

### Priority

```javascript
// Different channels for different data types
const controlChannel = peer.createDataChannel('/control', {
  ordered: true,
  priority: 'high'
});

const videoChannel = peer.createDataChannel('/video', {
  ordered: false,
  priority: 'low'
});
```

## Example: Local Control Surface

Browser-based controller talking directly to another browser:

```javascript
// Controller (browser 1)
const p2p = new ClaspP2P({
  signaling: 'ws://localhost:7330',
  clientId: 'controller'
});

await p2p.connect();
const display = await p2p.connectToPeer('display');

// Send control values directly to display
document.querySelector('#slider').addEventListener('input', (e) => {
  display.set('/brightness', parseFloat(e.target.value));
});

// Receiver (browser 2)
const p2p = new ClaspP2P({
  signaling: 'ws://localhost:7330',
  clientId: 'display'
});

await p2p.connect();

p2p.on('peer_connected', (peer) => {
  peer.on('/brightness', (value) => {
    document.body.style.opacity = value;
  });
});
```

## Example: Mesh Network

Multiple peers connected to each other:

```javascript
const p2p = new ClaspP2P({
  signaling: 'ws://localhost:7330',
  clientId: myId,
  mesh: true  // Connect to all discovered peers
});

await p2p.connect();

// Broadcast to all peers
p2p.broadcast('/announcement', { message: 'Hello everyone!' });

// Listen for broadcasts
p2p.on('broadcast', (data, fromPeer) => {
  console.log(`${fromPeer.id}: ${data.message}`);
});
```

## Connection States

Handle connection lifecycle:

```javascript
peer.on('connecting', () => {
  console.log('Establishing P2P connection...');
});

peer.on('connected', () => {
  console.log('P2P connection established');
});

peer.on('disconnected', () => {
  console.log('P2P connection lost');
  // Fall back to router
});

peer.on('failed', (error) => {
  console.error('P2P connection failed:', error);
});
```

## NAT Traversal

### STUN (Simple)

Works for most home networks:

```javascript
const rtcConfig = {
  iceServers: [
    { urls: 'stun:stun.l.google.com:19302' }
  ]
};
```

### TURN (Reliable)

Required for symmetric NATs:

```javascript
const rtcConfig = {
  iceServers: [
    { urls: 'stun:stun.l.google.com:19302' },
    {
      urls: 'turn:turn.example.com:3478',
      username: process.env.TURN_USER,
      credential: process.env.TURN_PASS
    }
  ],
  iceTransportPolicy: 'relay'  // Force TURN (for testing)
};
```

## Troubleshooting

### Connection Fails Immediately

1. Check signaling server is working
2. Verify both peers are connected to signaling
3. Check browser WebRTC support

### Connection Slow to Establish

1. Add more STUN servers
2. Check network allows UDP traffic
3. Consider adding TURN server

### Works Locally, Not Remotely

1. Add TURN server for NAT traversal
2. Check firewall settings
3. Verify STUN server is reachable

### High Latency Despite P2P

1. Verify P2P connection is active (not falling back to router)
2. Check ICE connection state
3. Network path may still be indirect

## Security

### Encryption

WebRTC DataChannels are encrypted by default (DTLS).

### Authentication

Verify peer identity through signaling:

```javascript
p2p.on('peer_connecting', async (peer) => {
  const verified = await verifyPeerIdentity(peer.id);
  if (!verified) {
    peer.reject('Identity verification failed');
  }
});
```

## Next Steps

- [Custom Bridge](custom-bridge.md)
- [Performance Tuning](performance-tuning.md)
- [Transport Reference](../../reference/transports/webrtc.md)
