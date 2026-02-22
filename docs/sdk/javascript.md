---
title: JavaScript SDK
description: Build CLASP clients with JavaScript and TypeScript
order: 1
---

# JavaScript SDK

The `@clasp-to/core` package provides a WebSocket client for connecting to a CLASP router from Node.js and browsers. It handles connection management, state operations, subscriptions with wildcard patterns, all five signal types, and automatic reconnection.

## Installation

```bash
npm install @clasp-to/core
```

## Connecting

Use `ClaspBuilder` to configure and connect a client:

```javascript
import { ClaspBuilder } from '@clasp-to/core';

const client = await new ClaspBuilder('ws://localhost:7330')
  .withName('My App')
  .withReconnect(true)
  .withReconnectInterval(3000)
  .withFeatures(['state', 'events'])
  .connect();

console.log('Connected, session:', client.session);
```

All builder methods are chainable. The builder also supports short-form aliases:

| Builder Method | Alias | Description |
|---|---|---|
| `.withName(name)` | `.name(name)` | Client display name |
| `.withToken(token)` | `.token(token)` | CPSK auth token |
| `.withReconnect(bool)` | `.reconnect(bool)` | Enable auto-reconnect |
| `.withReconnectInterval(ms)` | `.reconnectInterval(ms)` | Delay between reconnect attempts |
| `.withFeatures(list)` | `.features(list)` | Requested feature set |

`connect()` returns a `Promise<Clasp>` that resolves once the WebSocket handshake and CLASP HELLO exchange complete.

## Setting and Getting State

State is the primary data model in CLASP. Values set at an address persist on the router and are delivered to late joiners.

```javascript
// Write state
client.set('/lights/brightness', 0.8);
client.set('/lights/color', { r: 255, g: 100, b: 0 });

// Read state from the router (async round-trip)
const brightness = await client.get('/lights/brightness');
console.log(brightness); // 0.8

// Read from local cache (no network call, returns undefined if not cached)
const cached = client.cached('/lights/brightness');
```

`set()` is fire-and-forget. `get()` returns a `Promise` that resolves with the current value stored on the router. `cached()` returns the most recent value this client has seen for the address, or `undefined` if none.

## Subscriptions

Subscribe to addresses or wildcard patterns. Callbacks fire whenever a matching value changes.

```javascript
// Exact address
const unsub = client.on('/lights/brightness', (value, address) => {
  console.log('Brightness changed:', value);
});

// Single-level wildcard
client.on('/sensors/*', (value, address) => {
  console.log(address, '=', value);
});

// Multi-level wildcard
client.subscribe('/sensors/**', (value, address) => {
  console.log('Deep sensor:', address, value);
});

// Unsubscribe when done
unsub();
```

`on()` and `subscribe()` are equivalent. Both return an unsubscribe function.

### Subscription Options

Throttle high-frequency updates or filter insignificant changes:

```javascript
client.on('/sensors/accelerometer', (value, address) => {
  updateUI(value);
}, {
  maxRate: 30,    // max 30 callbacks per second
  epsilon: 0.01   // ignore changes smaller than 0.01
});
```

| Option | Type | Description |
|---|---|---|
| `maxRate` | `number` | Maximum callback invocations per second |
| `epsilon` | `number` | Minimum change threshold for numeric values |

## Signal Types

CLASP defines five signal types. `set()` handles persistent state (Param signals). The other four are for transient signals.

### Events

Fire-and-forget notifications. Not stored as state, not delivered to late joiners.

```javascript
client.emit('/alerts/motion-detected', { zone: 'lobby', confidence: 0.95 });
client.emit('/cues/go');  // payload is optional
```

### Streams

High-rate continuous data. Use `stream()` when the router should treat the signal as a continuous feed rather than discrete state changes.

```javascript
// Send audio level 60 times per second
setInterval(() => {
  client.stream('/audio/level', analyser.getLevel());
}, 1000 / 60);
```

### Gestures

Phased interaction signals with a lifecycle: `begin`, `update`, `end`.

```javascript
// Touch/drag interaction
canvas.addEventListener('pointerdown', (e) => {
  client.gesture('/input/drag', e.pointerId, 'begin', { x: e.clientX, y: e.clientY });
});

canvas.addEventListener('pointermove', (e) => {
  client.gesture('/input/drag', e.pointerId, 'update', { x: e.clientX, y: e.clientY });
});

canvas.addEventListener('pointerup', (e) => {
  client.gesture('/input/drag', e.pointerId, 'end');
});
```

The `id` parameter groups gesture phases. The router tracks active gestures and delivers all phases to subscribers.

### Timelines

Keyframe-based animations executed by the router.

```javascript
client.timeline('/lights/brightness', [
  { time: 0, value: 0.0 },
  { time: 1000, value: 1.0 },
  { time: 3000, value: 1.0 },
  { time: 4000, value: 0.0 }
], { loop: false, startTime: client.time() });
```

## Bundles

Group multiple operations into a single message. Bundles are delivered atomically.

```javascript
// Atomic bundle -- all values arrive together
client.bundle([
  { set: ['/lights/1/brightness', 0.8] },
  { set: ['/lights/2/brightness', 0.6] },
  { emit: ['/cues/scene-change', { scene: 'Act 2' }] }
]);

// Scheduled bundle -- execute at a specific server time
const twoSecondsFromNow = client.time() + 2_000_000; // microseconds
client.bundle([
  { set: ['/lights/1/brightness', 1.0] },
  { set: ['/lights/2/brightness', 1.0] }
], { at: twoSecondsFromNow });
```

`time()` returns the synchronized server time in microseconds.

## Events

Register callbacks for connection lifecycle events:

```javascript
client.onConnect(() => {
  console.log('Connected');
});

client.onDisconnect(() => {
  console.log('Disconnected');
});

client.onError((err) => {
  console.error('Error:', err);
});

client.onReconnect(() => {
  console.log('Reconnected');
});
```

## Auth

Pass a CPSK token to authenticate with the router:

```javascript
const client = await new ClaspBuilder('ws://localhost:7330')
  .withName('Secure Client')
  .withToken('cpsk_a1b2c3d4e5f6...')
  .connect();
```

The router validates the token during the HELLO handshake. If the token is invalid or lacks required scopes, `connect()` rejects. See [Auth](../auth/README.md) for token generation and scope configuration.

## Browser vs Node.js

The API is identical in both environments. The only difference is the underlying WebSocket implementation:

| Environment | WebSocket Provider |
|---|---|
| Browser | Native `WebSocket` API |
| Node.js | `ws` package (peer dependency) |

In the browser, import directly or use a bundler:

```html
<script type="module">
  import { ClaspBuilder } from '@clasp-to/core';

  const client = await new ClaspBuilder('ws://localhost:7330')
    .withName('Browser Client')
    .connect();

  client.on('/status/*', (value, address) => {
    document.getElementById('status').textContent = `${address}: ${value}`;
  });
</script>
```

## Value Types

All CLASP value types are mapped to JavaScript types:

| CLASP Type | JavaScript Type | Example |
|---|---|---|
| null | `null` | `null` |
| boolean | `boolean` | `true` |
| integer (i64) | `number` | `42` |
| float (f64) | `number` | `3.14` |
| string | `string` | `'hello'` |
| bytes | `Uint8Array` | `new Uint8Array([0x01, 0x02])` |
| array | `Array` | `[1, 'two', 3.0]` |
| object | `Object` | `{ r: 255, g: 0, b: 128 }` |

## Query

Inspect the router's current signal state:

```javascript
// Get all signals
const all = await client.getSignals();

// Query signals matching a pattern
const sensors = await client.querySignals('/sensors/**');
```

## Cleanup

Close the connection when your application exits:

```javascript
client.close();
```

After calling `close()`, the `connected` getter returns `false` and no further callbacks fire.

## Reconnection & Connection Lifecycle

The client manages reconnection automatically when `reconnect` is enabled (the default).

### Connection States

```
connecting --> connected --> disconnecting --> disconnected
                                                    |
                                                    v
                                              (auto-reconnect)
                                                    |
                                                    v
                                               connecting ...
```

### Auto-Reconnect Behavior

Auto-reconnect uses exponential backoff starting from the configured `reconnectInterval` (default: 5000ms), scaling by 1.5x per attempt up to a 30-second cap. After 10 failed attempts, reconnection stops and an error callback fires.

On successful reconnect, all existing subscriptions are automatically re-established and the router sends a fresh SNAPSHOT for matched params (late-joiner sync).

```javascript
const client = await new ClaspBuilder('ws://localhost:7330')
  .withName('Resilient App')
  .withReconnect(true)
  .withReconnectInterval(3000)
  .connect();

client.onDisconnect((reason) => {
  console.log('Lost connection:', reason);
  // Auto-reconnect kicks in automatically
});

client.onReconnect((attempt) => {
  console.log(`Reconnect attempt #${attempt}`);
});

client.onConnect(() => {
  console.log('Connected (or reconnected)');
});

client.onError((err) => {
  console.error('Connection error:', err.message);
});
```

### Manual Close

Call `close()` to disconnect without triggering auto-reconnect:

```javascript
client.close(); // Disables auto-reconnect, closes WebSocket
```

## Examples

Working examples in `examples/js/`:

| File | Description |
|------|-------------|
| `simple-publisher.js` | Basic value publishing |
| `simple-subscriber.js` | Subscribe to address patterns |
| `signal-types.js` | All five signal types in action |
| `bundles-and-scheduling.js` | Atomic and scheduled bundles |
| `gestures.js` | Touch, pen, and multi-touch input |
| `p2p-webrtc.js` | Peer-to-peer communication via WebRTC |
| `video-p2p.html` | P2P video with WebRTC PeerConnection |
| `video-relay.html` | Broadcast video via CLASP relay streams |
| `late-joiner.js` | Late-joiner state synchronization |
| `locks.js` | Distributed locking |
| `discovery.js` | Service discovery |
| `security-tokens.js` | CPSK token authentication |
| `embedded-server.js` | Embedded CLASP router in Node.js |

## Next Steps

- [Core Concepts](../concepts/architecture.md) -- understand signals, state, and the router model
- [Protocol Bridges](../protocols/README.md) -- connect CLASP to OSC, MIDI, MQTT, and more
- [Auth](../auth/README.md) -- CPSK tokens and capability delegation
- [P2P & WebRTC](../core/p2p.md) -- direct peer-to-peer connections
- [Python SDK](python.md) -- build CLASP clients with Python
- [Rust SDK](rust.md) -- build CLASP clients with Rust
