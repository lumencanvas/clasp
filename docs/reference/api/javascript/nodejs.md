# Node.js Usage

Node.js-specific notes for @clasp-to/core.

## Installation

```bash
npm install @clasp-to/core
```

## Quick Start

```javascript
const { Clasp } = require('@clasp-to/core');

async function main() {
  const client = await Clasp.connect('ws://localhost:7330');

  await client.set('/sensors/temp', 23.5);

  client.on('/control/**', (value, address) => {
    console.log(`${address}: ${value}`);
  });

  // Keep process running
  process.on('SIGINT', async () => {
    await client.disconnect();
    process.exit(0);
  });
}

main();
```

## ES Modules

```javascript
import { Clasp } from '@clasp-to/core';

const client = await Clasp.connect('ws://localhost:7330');
```

## CommonJS

```javascript
const { Clasp } = require('@clasp-to/core');

Clasp.connect('ws://localhost:7330').then(client => {
  // ...
});
```

## TypeScript

```typescript
import { Clasp, Value } from '@clasp-to/core';

const client: Clasp = await Clasp.connect('ws://localhost:7330');

interface SensorData {
  temp: number;
  humidity: number;
}

await client.set<SensorData>('/sensors', { temp: 23.5, humidity: 65 });
const data = await client.get<SensorData>('/sensors');
```

## Express Integration

```javascript
const express = require('express');
const { Clasp } = require('@clasp-to/core');

const app = express();
let clasp;

app.get('/api/value/:address', async (req, res) => {
  const value = await clasp.get('/' + req.params.address);
  res.json({ value });
});

app.post('/api/value/:address', express.json(), async (req, res) => {
  await clasp.set('/' + req.params.address, req.body.value);
  res.json({ success: true });
});

async function start() {
  clasp = await Clasp.connect('ws://localhost:7330');
  app.listen(3000, () => console.log('Server on port 3000'));
}

start();
```

## Server-Sent Events

Stream CLASP updates to browsers:

```javascript
const express = require('express');
const { Clasp } = require('@clasp-to/core');

const app = express();

app.get('/events/:pattern', async (req, res) => {
  const clasp = await Clasp.connect('ws://localhost:7330');
  const pattern = '/' + req.params.pattern.replace(/\./g, '/');

  res.setHeader('Content-Type', 'text/event-stream');
  res.setHeader('Cache-Control', 'no-cache');
  res.setHeader('Connection', 'keep-alive');

  const unsubscribe = clasp.on(pattern, (value, address) => {
    res.write(`data: ${JSON.stringify({ address, value })}\n\n`);
  });

  req.on('close', () => {
    unsubscribe();
    clasp.disconnect();
  });
});
```

## Socket.IO Integration

```javascript
const { Server } = require('socket.io');
const { Clasp } = require('@clasp-to/core');

const io = new Server(3000);

async function start() {
  const clasp = await Clasp.connect('ws://localhost:7330');

  io.on('connection', (socket) => {
    // Forward CLASP updates to Socket.IO
    const unsubscribe = clasp.on('/**', (value, address) => {
      socket.emit('clasp:update', { address, value });
    });

    // Forward Socket.IO commands to CLASP
    socket.on('clasp:set', async ({ address, value }) => {
      await clasp.set(address, value);
    });

    socket.on('disconnect', () => {
      unsubscribe();
    });
  });
}

start();
```

## Worker Threads

```javascript
const { Worker, isMainThread, parentPort } = require('worker_threads');
const { Clasp } = require('@clasp-to/core');

if (isMainThread) {
  // Main thread
  const worker = new Worker(__filename);

  worker.on('message', (msg) => {
    console.log('From worker:', msg);
  });

  worker.postMessage({ type: 'subscribe', pattern: '/sensors/**' });

} else {
  // Worker thread
  let clasp;

  parentPort.on('message', async (msg) => {
    if (msg.type === 'subscribe') {
      if (!clasp) {
        clasp = await Clasp.connect('ws://localhost:7330');
      }

      clasp.on(msg.pattern, (value, address) => {
        parentPort.postMessage({ address, value });
      });
    }
  });
}
```

## Cluster Mode

```javascript
const cluster = require('cluster');
const { Clasp } = require('@clasp-to/core');

if (cluster.isMaster) {
  // Fork workers
  for (let i = 0; i < 4; i++) {
    cluster.fork();
  }
} else {
  // Each worker connects independently
  const clasp = await Clasp.connect('ws://localhost:7330');

  // Handle requests
  // ...
}
```

## Graceful Shutdown

```javascript
const { Clasp } = require('@clasp-to/core');

let clasp;
let shuttingDown = false;

async function start() {
  clasp = await Clasp.connect('ws://localhost:7330');

  // Your application logic
}

async function shutdown() {
  if (shuttingDown) return;
  shuttingDown = true;

  console.log('Shutting down...');

  if (clasp) {
    await clasp.disconnect();
  }

  process.exit(0);
}

process.on('SIGINT', shutdown);
process.on('SIGTERM', shutdown);
process.on('uncaughtException', (err) => {
  console.error('Uncaught exception:', err);
  shutdown();
});

start();
```

## Error Handling

```javascript
const { Clasp } = require('@clasp-to/core');

process.on('unhandledRejection', (reason, promise) => {
  console.error('Unhandled Rejection:', reason);
});

async function main() {
  try {
    const clasp = await Clasp.builder('ws://localhost:7330')
      .withAutoReconnect(true)
      .connect();

    clasp.on('error', (err) => {
      console.error('CLASP error:', err);
    });

    // ...
  } catch (err) {
    console.error('Failed to connect:', err);
    process.exit(1);
  }
}

main();
```

## Environment Variables

```javascript
const { Clasp } = require('@clasp-to/core');

const client = await Clasp.builder(process.env.CLASP_URL || 'ws://localhost:7330')
  .withToken(process.env.CLASP_TOKEN)
  .withName(process.env.CLASP_CLIENT_NAME || 'node-client')
  .connect();
```

## Testing

```javascript
// Mock CLASP client for tests
const { MockClasp } = require('@clasp-to/core/testing');

describe('MyService', () => {
  let clasp;

  beforeEach(() => {
    clasp = new MockClasp();
  });

  it('should handle sensor data', async () => {
    const myService = new MyService(clasp);

    // Simulate CLASP message
    clasp.emit('/sensors/temp', 25.5);

    expect(myService.lastTemperature).toBe(25.5);
  });

  it('should send commands', async () => {
    const myService = new MyService(clasp);

    await myService.setLight(255);

    expect(clasp.lastSet).toEqual({
      address: '/lights/1',
      value: 255
    });
  });
});
```

## Logging

```javascript
const { Clasp, setLogLevel } = require('@clasp-to/core');

// Set log level
setLogLevel('debug'); // 'error' | 'warn' | 'info' | 'debug'

// Custom logger
setLogger({
  error: console.error,
  warn: console.warn,
  info: console.log,
  debug: () => {} // Disable debug
});
```

## Memory Management

```javascript
// Unsubscribe when done
const unsubscribe = clasp.on('/data/**', handler);
// ... later
unsubscribe();

// Close connection when done
await clasp.disconnect();

// Set can be garbage collected
clasp = null;
```

## See Also

- [@clasp-to/core API](clasp-core.md) - Full API reference
- [Browser Notes](browser.md) - Browser-specific usage
