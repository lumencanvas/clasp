---
title: "Node.js Usage"
description: "Node.js-specific notes for @clasp-to/core."
section: reference
order: 3
---
# Node.js Usage

Node.js-specific notes for @clasp-to/core.

## Installation

```bash
npm install @clasp-to/core
```

## Quick Start

### ES Modules

```javascript
import { ClaspBuilder } from '@clasp-to/core';

const client = await new ClaspBuilder('ws://localhost:7330')
  .withName('Node App')
  .connect();

client.set('/sensors/temp', 23.5);

client.on('/control/**', (value, address) => {
  console.log(`${address}: ${value}`);
});

// Graceful shutdown
process.on('SIGINT', () => {
  client.close();
  process.exit(0);
});
```

### CommonJS

```javascript
const { ClaspBuilder } = require('@clasp-to/core');

async function main() {
  const client = await new ClaspBuilder('ws://localhost:7330')
    .withName('Node App')
    .connect();

  // ...
}

main();
```

### TypeScript

```typescript
import { ClaspBuilder, Clasp, Value } from '@clasp-to/core';

const client: Clasp = await new ClaspBuilder('ws://localhost:7330')
  .withName('TS App')
  .connect();

client.set('/sensors', { temp: 23.5, humidity: 65 });
const data = await client.get('/sensors');
```

## Express Integration

```javascript
import express from 'express';
import { ClaspBuilder } from '@clasp-to/core';

const app = express();
app.use(express.json());

const clasp = await new ClaspBuilder('ws://localhost:7330')
  .withName('Express API')
  .connect();

app.get('/api/value/:address(*)', async (req, res) => {
  const value = await clasp.get('/' + req.params.address);
  res.json({ value });
});

app.post('/api/value/:address(*)', async (req, res) => {
  clasp.set('/' + req.params.address, req.body.value);
  res.json({ success: true });
});

app.listen(3000, () => console.log('Server on port 3000'));
```

## Server-Sent Events

Stream CLASP updates to browsers:

```javascript
import express from 'express';
import { ClaspBuilder } from '@clasp-to/core';

const app = express();

const clasp = await new ClaspBuilder('ws://localhost:7330')
  .withName('SSE Server')
  .connect();

app.get('/events/:pattern(*)', (req, res) => {
  const pattern = '/' + req.params.pattern.replace(/\./g, '/');

  res.setHeader('Content-Type', 'text/event-stream');
  res.setHeader('Cache-Control', 'no-cache');
  res.setHeader('Connection', 'keep-alive');

  const unsubscribe = clasp.on(pattern, (value, address) => {
    res.write(`data: ${JSON.stringify({ address, value })}\n\n`);
  });

  req.on('close', () => {
    unsubscribe();
  });
});
```

## Socket.IO Integration

```javascript
import { Server } from 'socket.io';
import { ClaspBuilder } from '@clasp-to/core';

const io = new Server(3000);

const clasp = await new ClaspBuilder('ws://localhost:7330')
  .withName('Socket.IO Bridge')
  .connect();

io.on('connection', (socket) => {
  // Forward CLASP updates to Socket.IO
  const unsubscribe = clasp.on('/**', (value, address) => {
    socket.emit('clasp:update', { address, value });
  });

  // Forward Socket.IO commands to CLASP
  socket.on('clasp:set', ({ address, value }) => {
    clasp.set(address, value);
  });

  socket.on('disconnect', () => {
    unsubscribe();
  });
});
```

## Worker Threads

```javascript
import { Worker, isMainThread, parentPort } from 'worker_threads';
import { ClaspBuilder } from '@clasp-to/core';

if (isMainThread) {
  const worker = new Worker(new URL(import.meta.url));

  worker.on('message', (msg) => {
    console.log('From worker:', msg);
  });

  worker.postMessage({ type: 'subscribe', pattern: '/sensors/**' });

} else {
  let clasp;

  parentPort.on('message', async (msg) => {
    if (msg.type === 'subscribe') {
      if (!clasp) {
        clasp = await new ClaspBuilder('ws://localhost:7330')
          .withName('Worker')
          .connect();
      }

      clasp.on(msg.pattern, (value, address) => {
        parentPort.postMessage({ address, value });
      });
    }
  });
}
```

## Graceful Shutdown

```javascript
import { ClaspBuilder } from '@clasp-to/core';

let clasp;
let shuttingDown = false;

async function start() {
  clasp = await new ClaspBuilder('ws://localhost:7330')
    .withName('My Service')
    .connect();

  // Your application logic
}

function shutdown() {
  if (shuttingDown) return;
  shuttingDown = true;

  console.log('Shutting down...');

  if (clasp) {
    clasp.close();
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
import { ClaspBuilder } from '@clasp-to/core';

process.on('unhandledRejection', (reason) => {
  console.error('Unhandled Rejection:', reason);
});

async function main() {
  try {
    const clasp = await new ClaspBuilder('ws://localhost:7330')
      .withName('My App')
      .withReconnect(true)
      .connect();

    clasp.onError((err) => {
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
import { ClaspBuilder } from '@clasp-to/core';

const builder = new ClaspBuilder(process.env.CLASP_URL || 'ws://localhost:7330')
  .withName(process.env.CLASP_CLIENT_NAME || 'node-client');

if (process.env.CLASP_TOKEN) {
  builder.withToken(process.env.CLASP_TOKEN);
}

const client = await builder.connect();
```

## Memory Management

```javascript
// Always unsubscribe when done
const unsubscribe = clasp.on('/data/**', handler);
// ... later
unsubscribe();

// Close connection when shutting down
clasp.close();
```

## See Also

- [@clasp-to/core API](clasp-core.md) - Full API reference
- [Browser Notes](browser.md) - Browser-specific usage
