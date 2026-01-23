# Signal Types

CLASP distinguishes signal types at the protocol level. This isn't just metadata—it affects routing, storage, reliability, and client behavior.

## Overview

| Type | Purpose | Default QoS | State? | Coalesce? |
|------|---------|-------------|--------|-----------|
| **Param** | Authoritative values | Confirm | Yes | Last value |
| **Event** | Triggers | Confirm | No | Never |
| **Stream** | High-rate data | Fire | No | Recent values |
| **Gesture** | Phased input | Fire | Phase only | By ID |
| **Timeline** | Time-indexed automation | Commit | Full | Never |

## Param (Stateful Values)

Params are the core state primitive. They represent values that:
- Have a current authoritative value
- Are tracked with revision numbers
- Support conflict resolution
- Are synchronized to late-joining clients

### Structure

```javascript
{
  address: "/app/scene/0/layer/3/opacity",
  value: 0.75,
  revision: 42,
  writer: "session:abc",
  timestamp: 1704067200000000,
  meta: {
    unit: "normalized",
    range: [0, 1],
    default: 1.0
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `address` | string | Signal address |
| `value` | any | Current value |
| `revision` | uint64 | Monotonic version number |
| `writer` | string | Session that wrote the value |
| `timestamp` | uint64 | When the value was set (µs) |
| `meta` | object | Optional metadata |

### Conflict Resolution

When multiple clients write simultaneously, the router resolves conflicts using one of these strategies:

| Strategy | Behavior | Use Case |
|----------|----------|----------|
| `lww` | Last-Write-Wins (by timestamp) | Default, simple |
| `max` | Keep maximum value | Meters, levels |
| `min` | Keep minimum value | Limits |
| `lock` | First writer holds lock | Exclusive control |
| `merge` | Application-defined | Complex objects |

### Locking

For exclusive control of a parameter:

```javascript
// Request lock
SET { address: "/mixer/fader/1", value: 0.5, lock: true }

// Router response if lock granted
ACK { address: "/mixer/fader/1", locked: true, holder: "session:abc" }

// Router response if lock denied
ERROR { code: 401, message: "Lock held", holder: "session:xyz" }

// Release lock
SET { address: "/mixer/fader/1", value: 0.5, unlock: true }
```

### Usage Examples

```typescript
// JavaScript
await client.set('/app/scene/0/opacity', 0.75);
const value = await client.get('/app/scene/0/opacity');

// With lock
await client.set('/mixer/fader/1', 0.5, { lock: true });
```

```python
# Python
await client.set('/app/scene/0/opacity', 0.75)
value = await client.get('/app/scene/0/opacity')
```

## Event (Triggers)

Events are ephemeral—they happen and are gone. They are:
- Not stored (no state)
- Delivered reliably (QoS Confirm by default)
- Never coalesced
- Used for triggers, cues, notifications

### Structure

```javascript
{
  address: "/app/cue/fire",
  payload: { cue: "intro", transition: "fade" },
  timestamp: 1704067200000000
}
```

### Usage Examples

```typescript
// JavaScript
await client.emit('/app/cue/fire', { cue: 'intro' });

client.on('/app/cue/*', (payload, address) => {
  console.log(`Cue fired: ${address}`, payload);
});
```

```python
# Python
await client.emit('/app/cue/fire', {'cue': 'intro'})

@client.on('/app/cue/*')
def on_cue(payload, address):
    print(f'Cue fired: {address}', payload)
```

## Stream (High-Rate Data)

Streams are for continuous data where occasional packet loss is acceptable. They are:
- Not stored
- Fire-and-forget (QoS Fire)
- May be coalesced or downsampled
- Used for faders, sensors, motion data

### Structure

```javascript
{
  address: "/controller/fader/1",
  samples: [0.50, 0.52, 0.55, 0.58],
  rate: 60,
  timestamp: 1704067200000000
}
```

| Field | Type | Description |
|-------|------|-------------|
| `samples` | array | Batched samples |
| `rate` | int | Sample rate in Hz |
| `timestamp` | uint64 | Timestamp of first sample |

### Subscription Options

```javascript
SUBSCRIBE {
  address: "/controller/fader/*",
  type: "stream",
  options: {
    maxRate: 30,      // Downsample to 30Hz
    epsilon: 0.01,    // Only send if change > 1%
    window: 100       // Buffer 100ms of samples
  }
}
```

### Usage Examples

```typescript
// JavaScript
client.stream('/controller/fader/1', 0.75);

client.on('/controller/fader/*', (value, address) => {
  updateUI(address, value);
}, { maxRate: 30, epsilon: 0.01 });
```

## Gesture (Phased Input)

Gestures are streams with semantic phases, designed for touch, pen, and motion input. They have:
- A stable ID for the gesture duration
- Phases: `start`, `move`, `end`, `cancel`
- Coalescing rules based on phase

### Structure

```javascript
{
  address: "/input/touch",
  id: 1,
  phase: "move",
  payload: {
    position: [0.5, 0.3],
    pressure: 0.8
  },
  timestamp: 1704067200000000
}
```

| Field | Type | Description |
|-------|------|-------------|
| `id` | int | Stable ID for this gesture |
| `phase` | string | `start`, `move`, `end`, `cancel` |
| `payload` | object | Gesture data |

### Phases

| Phase | Description | Coalesce? |
|-------|-------------|-----------|
| `start` | Gesture begins | Never |
| `move` | Gesture continues | Yes (keep recent) |
| `end` | Gesture ends normally | Never |
| `cancel` | Gesture cancelled | Never |

### Usage Examples

```typescript
// JavaScript
client.on('/input/touch', (gesture) => {
  switch (gesture.phase) {
    case 'start':
      beginDrag(gesture.id, gesture.payload.position);
      break;
    case 'move':
      updateDrag(gesture.id, gesture.payload.position);
      break;
    case 'end':
      endDrag(gesture.id);
      break;
  }
});
```

## Timeline (Automation)

Timelines are time-indexed sequences for automation, cues, and scheduling. They are:
- Fully stored
- Immutable once published
- Used for automation lanes, cue lists, scheduled changes

### Structure

```javascript
{
  address: "/app/scene/0/layer/3/opacity",
  type: "timeline",
  keyframes: [
    { time: 0, value: 1.0, easing: "linear" },
    { time: 1000000, value: 0.0, easing: "ease-out" }
  ],
  loop: false,
  startTime: 1704067200000000
}
```

| Field | Type | Description |
|-------|------|-------------|
| `keyframes` | array | Time-indexed values |
| `keyframes[].time` | uint64 | Time offset (µs) |
| `keyframes[].value` | any | Value at this time |
| `keyframes[].easing` | string | Easing function |
| `loop` | bool | Whether to loop |
| `startTime` | uint64 | When to begin playback |

### Easing Functions

Standard easing functions:
- `linear`
- `ease-in`, `ease-out`, `ease-in-out`
- `step` (hold until next keyframe)

## Signal Type Selection Guide

| Use Case | Signal Type |
|----------|-------------|
| Fader position | Param |
| Button press | Event |
| Accelerometer data | Stream |
| Touch interaction | Gesture |
| Lighting fade automation | Timeline |
| Configuration setting | Param |
| Notification/alert | Event |
| Audio level meter | Stream |
| Pen/stylus input | Gesture |
| Show cue list | Timeline |
