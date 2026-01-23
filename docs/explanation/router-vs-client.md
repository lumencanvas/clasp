# Router vs Client

Understanding the different roles in a CLASP system.

## Overview

CLASP has two fundamental roles:

| Role | Purpose | Typical Count |
|------|---------|---------------|
| **Router** | Central hub, state management, routing | One per system |
| **Client** | Application endpoint, sends/receives | Many per system |

## Router Responsibilities

The router is the brain of a CLASP system:

### Message Routing

Routes messages from publishers to subscribers based on address patterns:

```
Client A: SET /lights/1/brightness = 0.8
    │
    ▼
┌───────────────────────────────────────┐
│              Router                   │
│  ┌─────────────────────────────────┐  │
│  │      Pattern Matcher            │  │
│  │  Subscriptions:                 │  │
│  │    Client B: /lights/**         │  │
│  │    Client C: /lights/1/*        │  │
│  │    Client D: /audio/**          │  │
│  └─────────────────────────────────┘  │
└───────────────────────────────────────┘
    │           │
    ▼           ▼
Client B    Client C    (Client D doesn't match)
```

### State Management

Maintains authoritative state for all Parameters:

```
State Store:
├── /lights/1/brightness = 0.8  (rev: 42)
├── /lights/1/color = [255,0,0] (rev: 17)
├── /lights/2/brightness = 0.5  (rev: 8)
└── /audio/master/volume = 0.9  (rev: 3)
```

### Conflict Resolution

When multiple clients write simultaneously:

```
t=0: Client A sends SET /value = 10
t=0: Client B sends SET /value = 20
t=1: Router receives both
t=2: Router applies conflict resolution (e.g., last-write-wins)
t=3: Router broadcasts result to all subscribers
```

### Clock Source

Provides time synchronization for all clients:

```
Client                              Router
  │                                    │
  │── SYNC { t1 } ────────────────────►│
  │◄── SYNC { t1, t2, t3 } ────────────│
  │                                    │
  │ Calculate offset, sync local clock │
```

### Session Management

Tracks connected clients:

```
Sessions:
├── abc123: "Control Panel" (JavaScript)
├── def456: "Sensor Node" (Python)
└── ghi789: "Visualization" (Rust)
```

## Client Responsibilities

Clients are application endpoints:

### Connect and Authenticate

```javascript
const client = await new ClaspBuilder('ws://router:7330')
  .withName('My App')
  .withToken(authToken)
  .connect();
```

### Subscribe to Data

```javascript
// Subscribe to address patterns
client.on('/lights/**', (value, address) => {
  updateUI(address, value);
});
```

### Publish Data

```javascript
// Set parameters (stateful)
await client.set('/app/volume', 0.8);

// Emit events (ephemeral)
await client.emit('/app/cue/fire', { id: 1 });

// Stream data (high-rate)
client.stream('/app/motion/x', 0.5);
```

### Maintain Local Cache

```javascript
// Instant read from cache
const value = client.cached('/app/volume');

// Async read from router
const value = await client.get('/app/volume');
```

## Deployment Patterns

### Pattern 1: Standalone Router

Most common setup:

```
┌───────────────────┐
│   CLASP Router    │  (dedicated process)
│   localhost:7330  │
└─────────┬─────────┘
          │
    ┌─────┼─────┐
    │     │     │
┌───▼──┐ ┌▼────┐ ┌▼────┐
│App A │ │App B│ │App C│  (separate processes)
└──────┘ └─────┘ └─────┘
```

Start with CLI:
```bash
clasp server --port 7330
```

### Pattern 2: Embedded Router

Router runs inside your application:

```
┌────────────────────────────────────┐
│           Your App                 │
│  ┌──────────────────────────────┐  │
│  │       CLASP Router           │  │
│  │       (embedded)             │  │
│  └──────────────────────────────┘  │
│              ↑                     │
│    Direct access to state          │
└──────────────┬─────────────────────┘
               │
        ┌──────┼──────┐
        │      │      │
     ┌──▼──┐ ┌─▼──┐ ┌─▼──┐
     │Ext A│ │Ext B│ │Ext C│  (external clients)
     └─────┘ └────┘ └─────┘
```

```rust
// Rust embedded router
let router = Router::new(config);

// Direct state access
router.state().set_value("/app/status", Value::String("running"), "internal");

// External connections
router.serve_websocket("0.0.0.0:7330").await?;
```

### Pattern 3: Desktop App

The CLASP desktop app embeds a router:

```
┌────────────────────────────────────┐
│       CLASP Desktop App            │
│  ┌────────────────────────────┐    │
│  │      Embedded Router       │    │
│  └────────────────────────────┘    │
│  ┌────────────────────────────┐    │
│  │    Protocol Bridges        │    │
│  │  OSC  MIDI  MQTT  Art-Net  │    │
│  └────────────────────────────┘    │
└────────────────────────────────────┘
```

No separate router process needed.

## When to Use What

### Use Standalone Router When:

- Multiple independent applications need to communicate
- Router should survive application restarts
- Running in Docker/Kubernetes
- Need to scale router independently

### Use Embedded Router When:

- Single application is the primary data source
- Want direct state access without network
- Simplifying deployment
- Building a self-contained tool

## Can I Have Multiple Routers?

Yes, but carefully:

### Separate Systems

Different routers for different systems (no communication):

```
┌───────────┐     ┌───────────┐
│  Router A │     │  Router B │
│ (Lights)  │     │  (Audio)  │
└─────┬─────┘     └─────┬─────┘
      │                 │
  Clients           Clients
```

### Federated (Advanced)

Routers can be connected for larger deployments:

```
┌───────────┐     ┌───────────┐
│  Router A │◄───►│  Router B │
└───────────┘     └───────────┘
```

Federation is an advanced topic.

## Common Misconceptions

### "Bridges are routers"

No. Bridges are **clients** that also speak another protocol:

```
┌────────────────┐     ┌────────────────┐
│   OSC Bridge   │────►│    Router      │
│   (client)     │     │                │
└────────────────┘     └────────────────┘
```

The bridge connects TO the router, it doesn't route messages itself.

### "Every app needs a router"

No. Most apps only need a client. Only one router is needed per system.

### "Clients can route to each other"

No. All routing goes through the router. Clients only communicate with the router.

## Summary

| Aspect | Router | Client |
|--------|--------|--------|
| Count | One per system | Many |
| State | Authoritative | Cached |
| Routing | Yes | No |
| Clock | Source | Synced |
| Role | Infrastructure | Application |

Choose:
- **Standalone router** for multi-app systems
- **Embedded router** for single-app systems
- **Client** for everything else
