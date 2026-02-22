---
title: "Signals Not Messages"
description: "CLASP uses semantic signal types instead of generic messages. This design choice has profound implications for how the protocol works."
section: explanation
order: 11
---
# Signals Not Messages

CLASP uses semantic signal types instead of generic messages. This design choice has profound implications for how the protocol works.

## The Problem with Generic Messages

Most protocols treat all data the same way:

```javascript
// Generic message system
send({ type: "update", path: "/fader/1", value: 0.5 });
send({ type: "update", path: "/button/1", value: true });
send({ type: "update", path: "/motion/x", value: 0.123 });
```

But these have fundamentally different semantics:
- The fader has a **current value** that should persist
- The button is a **momentary trigger** that shouldn't be stored
- The motion data is a **continuous stream** where occasional loss is fine

Treating them the same causes problems:
- No way to ask "what's the current fader value?"
- Button presses get lost if unreliable transport
- Motion data floods with unnecessary reliability

## CLASP's Signal Types

CLASP distinguishes five signal types at the protocol level:

| Type | Semantics | Storage | Reliability | Example |
|------|-----------|---------|-------------|---------|
| **Param** | Current value | Persisted | Confirmed | Fader position |
| **Event** | Trigger | None | Confirmed | Button press |
| **Stream** | Continuous | None | Best-effort | Motion data |
| **Gesture** | Phased input | Phase only | Best-effort | Touch |
| **Timeline** | Automation | Full | Durable | Lighting cue |

## Why This Matters

### Smart Routing

The router can make intelligent decisions:

```
Param:   Store in state → Send to subscribers → ACK publisher
Event:   Forward to subscribers → ACK publisher → Discard
Stream:  Forward to subscribers → Done (no ACK, no storage)
```

### Appropriate Reliability

Each type gets the right reliability:

```javascript
// Param: Must be delivered and confirmed
await client.set('/mixer/volume', 0.8);  // Waits for ACK

// Event: Must be delivered
await client.emit('/cue/fire', { id: 1 });  // Waits for ACK

// Stream: Fire and forget
client.stream('/sensor/accel', [0.1, 0.2, 0.3]);  // Returns immediately
```

### State Queries

Only Params can be queried:

```javascript
// Works: Params have state
const volume = await client.get('/mixer/volume');

// Doesn't make sense: Events are ephemeral
// const button = await client.get('/button/1');  // No such thing

// Doesn't make sense: Streams have no "current value"
// const accel = await client.get('/sensor/accel');  // No such thing
```

### Late Joiner Synchronization

When a new client connects, it receives all current Params:

```javascript
client.on('/mixer/**', (value, address) => {
  // First call: SNAPSHOT with all current mixer params
  // Subsequent calls: Individual updates
});
```

Events, Streams, and Gestures aren't included—they're ephemeral.

### Rate Limiting

Streams can be intelligently downsampled:

```javascript
// Subscribe at reduced rate
client.on('/sensor/accel', callback, {
  maxRate: 30,      // Max 30 updates/second
  epsilon: 0.01     // Only if changed > 1%
});
```

This doesn't make sense for Events (you want every trigger) or Params (you want every state change).

### UI Hints

UI systems can render appropriately:

| Type | UI Widget |
|------|-----------|
| Param | Fader, knob, text field |
| Event | Button, trigger |
| Stream | Graph, meter |
| Gesture | Touch visualizer |
| Timeline | Automation lane |

## How Types Are Determined

### By Address Convention

Addresses can imply type:

```
/mixer/*/volume      → Param (state)
/cue/*               → Event (triggers)
/sensor/*/accel      → Stream (continuous)
/input/touch/*       → Gesture (phased)
/automation/*/lane/* → Timeline (scheduled)
```

### By Announcement

Clients can explicitly declare types:

```javascript
{
  type: "ANNOUNCE",
  signals: [
    { address: "/mixer/*/volume", type: "param" },
    { address: "/cue/*", type: "event" }
  ]
}
```

### By API Method

The API method determines type:

```javascript
client.set('/path', value);     // Param
client.emit('/path', payload);  // Event
client.stream('/path', value);  // Stream
```

## Comparison with Other Protocols

### OSC
OSC has no signal types—everything is a message:
- No way to query current value
- No state synchronization
- No reliability differentiation

### MQTT
MQTT has QoS levels but no semantic types:
- Retained messages approximate Params
- QoS 0/1/2 approximate reliability
- But no Stream/Gesture/Timeline concepts

### MIDI
MIDI has some semantic distinction:
- Notes vs CC vs SysEx
- But limited to musical concepts
- No networking or state management

## Design Implications

### Protocol Overhead

Signal type is encoded in the message:
- 4 bits in the flags byte
- No overhead for simple cases
- Enables smart routing

### API Design

APIs expose signal types explicitly:

```typescript
// TypeScript
interface Clasp {
  set(address: string, value: Value): Promise<void>;      // Param
  get(address: string): Promise<Value>;                   // Param
  emit(address: string, payload?: Value): Promise<void>;  // Event
  stream(address: string, value: Value): void;            // Stream
  on(pattern: string, callback: Callback): Unsubscribe;   // Any
}
```

### Bridge Translation

Bridges map external protocols to signal types:

| Protocol | Maps To |
|----------|---------|
| OSC message | Usually Param |
| MIDI CC | Param |
| MIDI Note On/Off | Event |
| MQTT retained | Param |
| MQTT non-retained | Event |
| Art-Net DMX | Stream (high-rate) or Param |

## Best Practices

### Choose the Right Type

| If your data... | Use |
|-----------------|-----|
| Has a "current value" | Param |
| Is a momentary trigger | Event |
| Is high-rate continuous | Stream |
| Has start/move/end phases | Gesture |
| Is time-indexed automation | Timeline |

### Don't Fight the Model

```javascript
// Wrong: Using Param for triggers
await client.set('/button/1', true);
await client.set('/button/1', false);

// Right: Using Event for triggers
await client.emit('/button/1', { pressed: true });
```

```javascript
// Wrong: Using Event for state
client.emit('/volume', 0.8);  // Lost if no subscribers!

// Right: Using Param for state
await client.set('/volume', 0.8);  // Stored, queryable
```

## Summary

Signal types aren't just metadata—they fundamentally change how data is handled:

- **Params** are state (stored, versioned, queryable)
- **Events** are triggers (delivered, not stored)
- **Streams** are continuous (best-effort, rate-limited)
- **Gestures** are phased (tracked by ID)
- **Timelines** are scheduled (executed at specific times)

This semantic distinction enables smart routing, appropriate reliability, and meaningful state management.
