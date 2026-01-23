# State Management

CLASP maintains authoritative state for all Parameters. This document explains how state works.

## The State Model

Every Parameter in CLASP has:

```javascript
{
  address: "/lights/main/brightness",
  value: 0.75,
  revision: 42,
  writer: "session:abc123",
  timestamp: 1704067200000000
}
```

| Field | Purpose |
|-------|---------|
| `address` | Unique identifier |
| `value` | Current value |
| `revision` | Version number (monotonically increasing) |
| `writer` | Session that last wrote |
| `timestamp` | When it was written (microseconds) |

## Where State Lives

State is maintained by the **router**:

```
┌─────────────────────────────────────────┐
│              Router                     │
│  ┌───────────────────────────────────┐  │
│  │          State Store              │  │
│  │  /lights/main/brightness = 0.75   │  │
│  │  /lights/main/color = [255,0,0]   │  │
│  │  /audio/master/volume = 0.9       │  │
│  │  ...                              │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
        │           │           │
        ▼           ▼           ▼
    Client A    Client B    Client C
    (local      (local      (local
     cache)      cache)      cache)
```

Clients may cache state locally, but the router is authoritative.

## State Operations

### SET

Update a parameter value:

```javascript
await client.set('/lights/main/brightness', 0.8);
```

What happens:
1. Client sends SET message with new value
2. Router validates (permissions, constraints)
3. Router updates state store
4. Router increments revision
5. Router broadcasts to subscribers
6. Router sends ACK to writer

### GET

Query current value:

```javascript
const brightness = await client.get('/lights/main/brightness');
```

What happens:
1. Client sends GET message
2. Router looks up in state store
3. Router sends SNAPSHOT response

### SNAPSHOT

Receive current state:

```javascript
// On connect, subscribers receive SNAPSHOT
client.on('/lights/**', (value, address) => {
  // First callback: all matching current values
  // Subsequent: individual updates
});
```

The SNAPSHOT message contains all parameters matching the subscription pattern.

## Revisions

Every SET increments the revision number:

```
SET /lights/brightness = 0.5  → revision 1
SET /lights/brightness = 0.6  → revision 2
SET /lights/brightness = 0.7  → revision 3
```

Revisions enable:

### Optimistic Locking

```javascript
// Read current value and revision
const { value, revision } = await client.get('/lights/brightness');

// Attempt update with expected revision
await client.set('/lights/brightness', newValue, { revision });

// Fails if someone else wrote in between
```

### Change Detection

```javascript
client.on('/lights/**', (value, address, meta) => {
  console.log(`${address} changed to ${value} (rev ${meta.revision})`);
});
```

### Conflict Resolution

When two clients write simultaneously, revisions help determine order.

## Late Joiner Synchronization

New clients automatically receive current state:

```javascript
// Client connects
const client = await new ClaspBuilder('ws://router:7330').connect();

// Subscribe (receives SNAPSHOT immediately)
client.on('/lights/**', (value, address) => {
  // Called with all current /lights/** values
  // Then called for each future update
});
```

This solves the "what's the current value?" problem that plagues protocols like OSC.

## State Scope

State only applies to Parameters, not other signal types:

| Signal Type | Has State? |
|-------------|------------|
| Param | Yes (stored, versioned) |
| Event | No (ephemeral) |
| Stream | No (ephemeral) |
| Gesture | Phase only |
| Timeline | Yes (full history) |

## State Persistence

By default, state is **in-memory**:
- Fast access
- Lost on router restart

For durability, routers can persist to disk:
- Survives restarts
- Higher write latency
- Use for important configuration

## State Size Limits

Practical limits:
- Address: 256 bytes max
- Value: 64KB max (recommended: < 1KB)
- Total state: Limited by router memory

For large data, store references not values:
```javascript
// Don't do this
await client.set('/video/frame', hugeBuffer);

// Do this
await client.set('/video/frame/url', 'http://server/frames/123');
```

## Namespace Isolation

State can be partitioned by namespace:

```
/app1/**  → Visible to app1 clients
/app2/**  → Visible to app2 clients
/shared/** → Visible to all
```

This is controlled by capability tokens.

## State and Subscriptions

Subscriptions create a flow:

```
1. Client subscribes to /lights/**
2. Router sends SNAPSHOT of all matching params
3. Router registers subscription
4. Future SETs to matching addresses forwarded
5. Client unsubscribes
6. Router removes subscription (no more updates)
```

## Caching

Clients can cache state locally:

```javascript
// Cached read (instant, might be stale)
const value = client.cached('/lights/brightness');

// Fresh read (network round-trip)
const value = await client.get('/lights/brightness');
```

Cache is updated automatically when subscribed.

## State Transfer

State can be exported and imported:

```javascript
// Export all state
const snapshot = await client.get('/**');

// Import to new router
for (const param of snapshot) {
  await newClient.set(param.address, param.value);
}
```

## Best Practices

### Don't Store Large Values

```javascript
// Bad: Large binary data
await client.set('/video/frame', largeBuffer);

// Good: Reference to external storage
await client.set('/video/frame/url', 'http://storage/frame.png');
```

### Use Appropriate Types

```javascript
// Bad: Using Param for momentary triggers
await client.set('/button', true);
await client.set('/button', false);  // State is "false" forever

// Good: Using Event
await client.emit('/button');  // No state stored
```

### Structure Addresses Meaningfully

```javascript
// Bad: Flat namespace
/brightness
/color
/x
/y

// Good: Hierarchical
/lights/main/brightness
/lights/main/color
/position/x
/position/y
```

### Use Wildcards for Bulk Operations

```javascript
// Get all light parameters at once
const lights = await client.get('/lights/**');

// Subscribe to all mixer channels
client.on('/mixer/channel/*/**', handleChannelUpdate);
```

## See Also

- [Conflict Resolution](conflict-resolution.md) — Handling concurrent writes
- [Signal Types](signals-not-messages.md) — When to use Params vs Events
- [Addressing](../reference/protocol/addressing.md) — Address format and wildcards
