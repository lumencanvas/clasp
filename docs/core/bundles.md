---
title: Bundles
description: Atomic and scheduled signal bundles
order: 4
---

# Bundles

Bundles group multiple signals into a single unit. They can be atomic (all signals delivered together) or scheduled (signals execute at a future time). Bundles are the mechanism for coordinated, transactional updates in CLASP.

## Atomic Bundles

An atomic bundle delivers multiple operations as one unit. If any operation fails (e.g., a lock conflict or auth rejection), none of the operations are applied. This is essential when multiple values must be consistent -- for example, setting RGB channels of a light simultaneously.

**JavaScript:**

```javascript
// Set all three color channels atomically
client.bundle([
  { set: ['/lights/room1/r', 255] },
  { set: ['/lights/room1/g', 0] },
  { set: ['/lights/room1/b', 128] },
]);
```

**Python:**

```python
# Set all three color channels atomically
await client.bundle([
    {'set': ['/lights/room1/r', 255]},
    {'set': ['/lights/room1/g', 0]},
    {'set': ['/lights/room1/b', 128]},
])
```

Subscribers see all three values update at the same time. There is no intermediate state where only one or two channels have changed.

## Mixing Signal Types

Bundles can contain any combination of signal types -- SETs, events, streams, and more:

```javascript
client.bundle([
  { set: ['/lights/stage/brightness', 1.0] },
  { set: ['/lights/stage/color', [255, 200, 100]] },
  { emit: ['/cues/go', { scene: 'finale' }] },
]);
```

This sets the stage brightness and color, then fires a cue trigger, all in one atomic operation.

## Scheduled Bundles

A scheduled bundle includes a timestamp that tells the router when to execute the operations. The timestamp is in microseconds of server time, obtained via clock synchronization. This allows precise coordination across multiple clients -- everyone executes the same bundle at the same moment.

**JavaScript:**

```javascript
// Execute 5 seconds from now
const future = client.time() + 5_000_000;
client.bundle([
  { set: ['/lights/stage/brightness', 1.0] },
  { emit: ['/cues/go', { scene: 'finale' }] },
], { at: future });
```

**Python:**

```python
# Execute 5 seconds from now
future = client.time() + 5_000_000
await client.bundle([
    {'set': ['/lights/stage/brightness', 1.0]},
    {'emit': ['/cues/go', {'scene': 'finale'}]},
], at=future)
```

The `client.time()` method returns the current synchronized server time in microseconds. Adding an offset gives you a future execution time. All clients that have synchronized their clocks with the router will see the bundle execute at the same wall-clock moment.

## Bundle QoS

Bundles use **Commit** (QoS 2, exactly-once ordered delivery) by default. This is the strongest delivery guarantee in CLASP and ensures:

- The bundle arrives exactly once (no duplicates).
- Operations within the bundle are applied in order.
- Either all operations succeed or none are applied.

This default cannot be downgraded -- bundles always use Commit QoS.

## Wire Format

On the wire, a bundle is a single `BUNDLE` message (type code `0x30`) containing:

| Field | Type | Description |
|-------|------|-------------|
| `timestamp` | u64 or null | Scheduled execution time in microseconds (null for immediate) |
| `messages` | array | Array of `Message` objects (SET, PUBLISH, etc.) |

The entire bundle is encoded in a single frame, ensuring atomic delivery at the transport level.

## Use Cases

**Lighting scenes** -- set multiple fixture properties atomically so there are no partial states:

```javascript
client.bundle([
  { set: ['/lights/spot1/brightness', 1.0] },
  { set: ['/lights/spot1/color', [255, 220, 180]] },
  { set: ['/lights/spot2/brightness', 0.5] },
  { set: ['/lights/spot2/color', [100, 100, 255]] },
  { set: ['/lights/wash/brightness', 0.3] },
]);
```

**Timed cues** -- synchronize cue execution across distributed systems:

```javascript
const cueTime = client.time() + 10_000_000; // 10 seconds from now
client.bundle([
  { emit: ['/cues/go', { scene: 'act2' }] },
  { set: ['/show/state', 'act2'] },
], { at: cueTime });
```

**Transactional updates** -- ensure all-or-nothing state changes when multiple params must stay consistent:

```javascript
client.bundle([
  { set: ['/game/player/1/position', { x: 10, y: 20 }] },
  { set: ['/game/player/1/health', 80] },
  { set: ['/game/player/1/inventory', ['sword', 'shield']] },
]);
```

## Next Steps

- [Signal Types](./signals.md) -- the 5 signal types that can be included in bundles.
- [State Management](./state.md) -- how param state is updated within atomic bundles.
- [Addressing](./addressing.md) -- how bundle operations target specific addresses.
