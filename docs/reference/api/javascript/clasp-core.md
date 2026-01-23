# @clasp-to/core (JavaScript)

CLASP client library for JavaScript and TypeScript.

## Overview

`@clasp-to/core` provides a full-featured CLASP client for Node.js and browsers.

```bash
npm install @clasp-to/core
```

## Quick Start

```javascript
const { Clasp } = require('@clasp-to/core');

async function main() {
  const client = await Clasp.connect('ws://localhost:7330');

  // Set a value
  await client.set('/sensors/temp', 23.5);

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
import { Clasp, Value } from '@clasp-to/core';

const client: Clasp = await Clasp.connect('ws://localhost:7330');

interface SensorData {
  temperature: number;
  humidity: number;
}

await client.set<SensorData>('/sensors/data', {
  temperature: 23.5,
  humidity: 65
});

const data = await client.get<SensorData>('/sensors/data');
```

## Connection

### Basic Connection

```javascript
const client = await Clasp.connect('ws://localhost:7330');
```

### Builder Pattern

```javascript
const client = await Clasp.builder('ws://localhost:7330')
  .withName('my-client')
  .withTimeout(10000)
  .connect();
```

### With Authentication

```javascript
const client = await Clasp.builder('wss://router.example.com:7330')
  .withToken('eyJhbGciOi...')
  .connect();
```

### Auto-Discovery

```javascript
// Discover and connect to first available router
const client = await Clasp.discover();

// With preferences
const client = await Clasp.discover({
  timeout: 5000,
  preferName: 'Studio Router'
});
```

### Auto-Reconnect

```javascript
const client = await Clasp.builder('ws://localhost:7330')
  .withAutoReconnect(true)
  .withReconnectInterval(1000)
  .withMaxReconnectAttempts(10)
  .connect();
```

## Core Operations

### set(address, value)

Set a value at an address:

```javascript
// Primitives
await client.set('/path/to/value', 42);
await client.set('/path/to/value', 'hello');
await client.set('/path/to/value', true);
await client.set('/path/to/value', 3.14);

// Objects
await client.set('/path/to/value', { x: 1, y: 2 });

// Arrays
await client.set('/path/to/value', [1, 2, 3]);

// Null
await client.set('/path/to/value', null);
```

### get(address)

Get a value:

```javascript
const value = await client.get('/path/to/value');

// With timeout
const value = await client.get('/path/to/value', { timeout: 5000 });

// Returns undefined if not found
const value = await client.get('/nonexistent');
console.log(value); // undefined
```

### emit(address, value)

Emit an event (ephemeral, not stored):

```javascript
await client.emit('/events/button_pressed', { button: 1 });
await client.emit('/events/ping', null);
```

### delete(address)

Delete a value:

```javascript
await client.delete('/path/to/value');
```

### list(pattern)

List addresses matching a pattern:

```javascript
const addresses = await client.list('/sensors/**');
// ['/sensors/temp', '/sensors/humidity', '/sensors/room1/temp', ...]
```

## Subscriptions

### on(pattern, callback)

Subscribe to address pattern:

```javascript
// Single address
client.on('/sensors/temp', (value) => {
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
```

### Subscription Options

```javascript
client.on('/sensors/temp', handler, {
  maxRate: 30,          // Max 30 updates/sec
  debounce: 100,        // Wait 100ms after last update
  includeInitial: true  // Receive current value immediately
});
```

### Unsubscribe

```javascript
// Store unsubscribe function
const unsubscribe = client.on('/sensors/temp', handler);

// Unsubscribe when done
unsubscribe();
```

### once(pattern)

Wait for single value:

```javascript
const value = await client.once('/events/ready');
```

## Bundles

Atomic multi-operation bundles:

```javascript
// Using array syntax
await client.bundle([
  { set: ['/lights/1', 255] },
  { set: ['/lights/2', 128] },
  { emit: ['/cue/fired', { cue: 1 }] }
]);

// Using builder
await client.bundle()
  .set('/lights/1', 255)
  .set('/lights/2', 128)
  .emit('/cue/fired', { cue: 1 })
  .execute();
```

### Scheduled Bundles

```javascript
// Execute 5 seconds in the future
await client.bundle()
  .set('/lights/1', 255)
  .atTime(Date.now() + 5000)
  .execute();
```

## Streams

For high-rate continuous data:

```javascript
// Create stream
const stream = client.stream('/audio/level');

// Send values at high rate
setInterval(() => {
  stream.send(getAudioLevel());
}, 10);

// Stop stream
stream.stop();
```

## Gestures

For phased interactions:

```javascript
// Begin gesture
const gesture = client.gestureBegin('/draw/stroke', { x: 100, y: 100 });

// Update during gesture
gesture.update({ x: 150, y: 120 });
gesture.update({ x: 200, y: 150 });

// End gesture
gesture.end({ x: 250, y: 180 });
```

## Connection Events

```javascript
client.on('connected', () => {
  console.log('Connected');
});

client.on('disconnected', (reason) => {
  console.log('Disconnected:', reason);
});

client.on('reconnecting', (attempt) => {
  console.log('Reconnecting, attempt', attempt);
});

client.on('error', (error) => {
  console.error('Error:', error);
});
```

## Connection State

```javascript
// Check connection
if (client.isConnected()) {
  // ...
}

// Wait for connection
await client.waitConnected();

// Ping
const latency = await client.ping();
console.log(`Latency: ${latency}ms`);
```

## Clock Synchronization

```javascript
// Sync clock with router
await client.syncClock();

// Get synchronized time
const time = client.syncedTime();
```

## Locks

```javascript
// Acquire lock
const lock = await client.lock('/exclusive/resource');

try {
  // Use locked resource
  await client.set('/exclusive/resource', value);
} finally {
  // Release lock
  await lock.release();
}
```

## Disconnect

```javascript
await client.disconnect();
```

## Error Handling

```javascript
try {
  await client.get('/path');
} catch (error) {
  if (error.code === 'NOT_FOUND') {
    console.log('Not found');
  } else if (error.code === 'PERMISSION_DENIED') {
    console.log('Access denied');
  } else if (error.code === 'TIMEOUT') {
    console.log('Request timed out');
  } else {
    console.error('Error:', error);
  }
}
```

## See Also

- [Browser Notes](browser.md) - Browser-specific usage
- [Node.js Notes](nodejs.md) - Node.js-specific usage
- [JavaScript Installation](../../../how-to/installation/javascript-library.md)
