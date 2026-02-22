---
title: First Connection
description: Send your first signal through CLASP in 5 minutes
order: 2
---

# First Connection

This guide walks you through connecting two clients to a CLASP router: one publishing sensor data, one subscribing to it. By the end, you will have real-time data flowing between a JavaScript publisher and a Python subscriber.

## Prerequisites

- Node.js 18+ or Python 3.9+
- A CLASP router running (see [Installation](../getting-started/README.md))

## Architecture

```
Publisher (JS)  ──>  Router (port 7330)  ──>  Subscriber (Python)
   set()                                        on()
```

The router is the central hub. Publishers write state with `set()`, subscribers receive updates with `on()`. The router handles routing, state storage, and pattern matching.

## Start the Router

Using the CLI:

```bash
clasp server
```

Or with Docker:

```bash
docker run -p 7330:7330 ghcr.io/lumencanvas/clasp-relay
```

You should see output indicating the router is listening on port 7330.

## Publisher (JavaScript)

Create a file called `publisher.mjs`:

```javascript
import { ClaspBuilder } from '@clasp-to/core';

const client = await new ClaspBuilder('ws://localhost:7330')
  .withName('Sensor')
  .connect();

// Publish temperature every second
setInterval(() => {
  const temp = 20 + Math.random() * 10;
  client.set('/sensors/temperature', temp);
  console.log('Published:', temp.toFixed(1));
}, 1000);
```

Run it:

```bash
npm install @clasp-to/core
node publisher.mjs
```

## Subscriber (Python)

Create a file called `subscriber.py`:

```python
import asyncio
from clasp import Clasp

async def main():
    client = Clasp('ws://localhost:7330', name='Monitor')

    @client.on('/sensors/*')
    def on_sensor(value, address):
        print(f'{address}: {value}')

    await client.connect()
    await asyncio.Event().wait()  # run forever

asyncio.run(main())
```

Run it:

```bash
pip install clasp-to
python subscriber.py
```

You should see temperature readings printing in the subscriber terminal as the publisher sends them.

## What Just Happened?

1. The **router** started and began accepting WebSocket connections on port 7330.
2. The **publisher** connected, then called `set('/sensors/temperature', value)` every second. This updates the state at that address and notifies all subscribers.
3. The **subscriber** connected with a wildcard pattern `/sensors/*`, which matches any single-level child under `/sensors/`. Every time the publisher sets a value, the subscriber callback fires.
4. If the subscriber connects *after* the publisher has already sent values, it receives the **current state** immediately on subscription. This is late-joiner sync -- no messages are missed.

## Try These Experiments

| Experiment | Code |
|---|---|
| Add a second sensor | `client.set('/sensors/humidity', 65.2)` |
| Get current value | `await client.get('/sensors/temperature')` |
| Use events instead of state | `client.emit('/alerts/high-temp', { temp: 35 })` |
| Subscribe more specifically | `on('/sensors/temperature')` instead of `on('/sensors/*')` |

Events (via `emit()`) are fire-and-forget -- they are not stored as state and will not be delivered to late joiners. Use `set()` when you need persistence, `emit()` when you need notifications.

## Understanding Addresses

CLASP uses hierarchical addresses with wildcard support:

| Pattern | Matches | Example Match |
|---|---|---|
| `/sensors/temperature` | Exact address only | `/sensors/temperature` |
| `/sensors/*` | Any single level under `/sensors/` | `/sensors/temperature`, `/sensors/humidity` |
| `/sensors/**` | Any depth under `/sensors/` | `/sensors/temperature`, `/sensors/room/1/temperature` |

Addresses are forward-slash-delimited paths. Wildcards only apply to subscriptions, not to `set()` or `emit()` targets.

## Troubleshooting

| Problem | Cause | Fix |
|---|---|---|
| Connection refused | Router is not running | Start the router with `clasp server` or `docker run -p 7330:7330 clasp-relay` |
| Module not found (`@clasp-to/core`) | Package not installed | Run `npm install @clasp-to/core` |
| Module not found (`clasp`) | Package not installed | Run `pip install clasp-to` |
| No values received | Address pattern does not match | Check that the subscriber pattern matches the publisher address (e.g., `/sensors/*` matches `/sensors/temperature`) |
| Values received only once | Using `emit()` on publisher, `on()` on subscriber | `emit()` sends events, not state updates. Use `set()` for continuous state updates |

## Next Steps

- [JavaScript SDK](../sdk/javascript.md) -- full API reference for `@clasp-to/core`
- [Python SDK](../sdk/python.md) -- full API reference for `clasp-to`
- [Signal Types](../core/signals.md) -- understand Param, Event, Stream, Gesture, and Timeline
- [Deployment](../deployment/relay.md) -- run CLASP in production with auth and persistence
