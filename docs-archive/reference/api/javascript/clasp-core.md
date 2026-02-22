---
title: "@clasp-to/core (JavaScript)"
description: "CLASP client library for JavaScript and TypeScript."
section: reference
order: 2
---
# @clasp-to/core (JavaScript)

CLASP client library for JavaScript and TypeScript.

## Overview

`@clasp-to/core` provides a full-featured CLASP client for Node.js and browsers.

```bash
npm install @clasp-to/core
```

## Quick Start

```javascript
import { ClaspBuilder } from '@clasp-to/core';

async function main() {
  const client = await new ClaspBuilder('ws://localhost:7330')
    .withName('My App')
    .connect();

  // Set a value
  client.set('/sensors/temp', 23.5);

  // Get a value
  const value = await client.get('/sensors/temp');
  console.log('Temperature:', value);

  // Subscribe to changes
  client.on('/sensors/**', (value, address) => {
    console.log(`${address}: ${value}`);
  });
}

main();
```

## TypeScript

Full TypeScript support included:

```typescript
import { ClaspBuilder, Clasp, Value } from '@clasp-to/core';

const client: Clasp = await new ClaspBuilder('ws://localhost:7330')
  .withName('my-app')
  .connect();

client.set('/sensors/data', { temperature: 23.5, humidity: 65 });
const data = await client.get('/sensors/data');
```

## Connection

### Builder Pattern

```javascript
const client = await new ClaspBuilder('ws://localhost:7330')
  .withName('my-client')
  .connect();
```

### With Authentication

```javascript
const client = await new ClaspBuilder('wss://router.example.com:7330')
  .withName('my-client')
  .withToken('eyJhbGciOi...')
  .connect();
```

### Auto-Reconnect

Reconnection is enabled by default. Configure it through the builder:

```javascript
const client = await new ClaspBuilder('ws://localhost:7330')
  .withName('my-client')
  .withReconnect(true)
  .withReconnectInterval(1000)
  .connect();
```

## Core Operations

### set(address, value)

Set a persistent parameter value at an address:

```javascript
// Primitives
client.set('/path/to/value', 42);
client.set('/path/to/value', 'hello');
client.set('/path/to/value', true);
client.set('/path/to/value', 3.14);

// Objects
client.set('/path/to/value', { x: 1, y: 2 });

// Arrays
client.set('/path/to/value', [1, 2, 3]);

// Null
client.set('/path/to/value', null);
```

> [!NOTE]
> `set()` is fire-and-forget (void). It does not return a Promise.

### get(address)

Get the current value at an address:

```javascript
const value = await client.get('/path/to/value');

// Returns undefined if the address has no value
const value = await client.get('/nonexistent');
console.log(value); // undefined
```

### emit(address, payload?)

Emit an event (ephemeral, not stored):

```javascript
client.emit('/events/button_pressed', { button: 1 });
client.emit('/events/ping');
```

> [!NOTE]
> `emit()` is fire-and-forget (void). Events are not stored — only currently subscribed clients receive them.

### stream(address, value)

Send high-rate continuous data (QoS.Fire):

```javascript
// Send a single stream frame
client.stream('/audio/level', getAudioLevel());

// High-rate usage
setInterval(() => {
  client.stream('/sensor/position', { x: getX(), y: getY() });
}, 10);
```

### gesture(address, gestureId, phase, payload?)

Send phased interactions (touch, drawing, etc.):

```javascript
const gestureId = 'touch-1';

// Begin gesture
client.gesture('/draw/stroke', gestureId, 'begin', { x: 100, y: 100 });

// Update during gesture
client.gesture('/draw/stroke', gestureId, 'update', { x: 150, y: 120 });
client.gesture('/draw/stroke', gestureId, 'update', { x: 200, y: 150 });

// End gesture
client.gesture('/draw/stroke', gestureId, 'end', { x: 250, y: 180 });
```

### timeline(address, keyframes, options?)

Send keyframe animation data:

```javascript
client.timeline('/lights/fade', [
  { time: 0, value: 0 },
  { time: 1000, value: 255 },
  { time: 2000, value: 0 }
]);
```

## Subscriptions

### on(pattern, callback) / subscribe(pattern, callback)

Subscribe to an address pattern. Returns an unsubscribe function:

```javascript
// Single address
const unsub = client.on('/sensors/temp', (value) => {
  console.log('Temperature:', value);
});

// Wildcard - single segment
client.on('/sensors/*/temp', (value, address) => {
  console.log(`${address}: ${value}`);
});

// Wildcard - multiple segments
client.on('/sensors/**', (value, address) => {
  console.log(`${address}: ${value}`);
});

// Unsubscribe when done
unsub();
```

`subscribe()` is an alias for `on()`.

### cached(address)

Get the locally cached value (synchronous, no network request):

```javascript
const value = client.cached('/sensors/temp');
// Returns undefined if no cached value
```

## Bundles

Atomic multi-operation bundles:

```javascript
await client.bundle([
  { set: ['/lights/1', 255] },
  { set: ['/lights/2', 128] },
  { emit: ['/cue/fired', { cue: 1 }] }
]);

// With options (e.g., scheduled execution)
await client.bundle([
  { set: ['/lights/1', 255] },
  { set: ['/lights/2', 128] }
], { time: client.time() + 5000000 }); // 5 seconds in future (microseconds)
```

## Signal Discovery

### getSignals()

Get all announced signal definitions:

```javascript
const signals = client.getSignals();
// [{ address, type, meta }, ...]
```

### querySignals(pattern)

Match signals by address pattern:

```javascript
const sensorSignals = client.querySignals('/sensors/**');
```

## Connection State

### Properties

```javascript
// Check connection status
if (client.connected) {
  console.log('Connected');
}

// Get session ID
console.log('Session:', client.session);

// Get server time (microseconds)
const serverTime = client.time();
```

### Connection Events

```javascript
client.onConnect(() => {
  console.log('Connected');
});

client.onDisconnect(() => {
  console.log('Disconnected');
});

client.onReconnect(() => {
  console.log('Reconnected');
});

client.onError((error) => {
  console.error('Error:', error);
});
```

## Disconnect

```javascript
client.close();
```

## Error Handling

```javascript
try {
  const value = await client.get('/path');
} catch (error) {
  console.error('Error:', error);
}
```

## See Also

- [Browser Notes](browser.md) - Browser-specific usage
- [Node.js Notes](nodejs.md) - Node.js-specific usage
- [JavaScript Installation](../../../how-to/installation/javascript-library.md)
