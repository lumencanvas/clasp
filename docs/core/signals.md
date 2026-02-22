---
title: Signal Types
description: The 5 signal types and when to use each
order: 1
---

# Signal Types

CLASP uses 5 signal types instead of generic messages. Each type has distinct semantics for storage, delivery, and routing. Choosing the right signal type gives you correct default behavior for QoS, persistence, and late-joiner sync without extra configuration.

## Overview

| Type | Semantics | Stored? | Default QoS | Example |
|------|-----------|---------|-------------|---------|
| Param | Current value | Yes | Confirm (1) | Fader position |
| Event | One-shot trigger | No | Confirm (1) | Button press |
| Stream | Continuous data | No | Fire (0) | Motion capture |
| Gesture | Phased input | Phase only | Fire (0) | Touch/pen |
| Timeline | Keyframe automation | Full | Commit (2) | Lighting cue |

## Param

A param represents a stateful value that persists across sessions. Every write increments a monotonic revision counter, and the router tracks the writer's session ID and a microsecond timestamp. When a new client subscribes to a param's address, it receives a SNAPSHOT of the current value immediately -- no state is missed.

Conflict resolution strategies (LWW, max, min, lock, merge) apply to params. See [State Management](./state.md) for details.

**JavaScript:**

```javascript
// Set a param value
client.set('/lights/room1/brightness', 0.75);

// Read current value
const brightness = await client.get('/lights/room1/brightness');

// Subscribe to changes
client.on('/lights/room1/brightness', (value, meta) => {
  console.log(`brightness: ${value} (rev ${meta.revision})`);
});
```

**Python:**

```python
# Set a param value
await client.set('/lights/room1/brightness', 0.75)

# Read current value
brightness = await client.get('/lights/room1/brightness')

# Subscribe to changes
@client.on('/lights/room1/brightness')
async def on_brightness(value, meta):
    print(f"brightness: {value} (rev {meta.revision})")
```

## Event

An event is an ephemeral trigger that is not stored. Subscribers receive events in real time, but late joiners do not receive past events. Use events for things that happen once and do not have a "current value."

**JavaScript:**

```javascript
// Emit an event
client.emit('/cues/go', { scene: 'intro' });

// Subscribe to events
client.on('/cues/go', (payload) => {
  console.log(`cue triggered: ${payload.scene}`);
});
```

**Python:**

```python
# Emit an event
await client.emit('/cues/go', {'scene': 'intro'})

# Subscribe to events
@client.on('/cues/go')
async def on_cue(payload):
    print(f"cue triggered: {payload['scene']}")
```

When to use events: button presses, cue triggers, notifications, one-shot commands.

## Stream

A stream carries high-rate continuous data with best-effort delivery (Fire QoS). Streams are not stored -- if a value is dropped, the next one replaces it. This makes streams suitable for data where freshness matters more than completeness.

**JavaScript:**

```javascript
// Send stream data
client.stream('/sensors/accelerometer', { x: 0.1, y: -0.3, z: 9.8 });

// Subscribe with downsampling
client.on('/sensors/accelerometer', (value) => {
  updateVisualization(value);
}, { maxRate: 30, epsilon: 0.01 });
```

**Python:**

```python
# Send stream data
await client.stream('/sensors/accelerometer', {'x': 0.1, 'y': -0.3, 'z': 9.8})

# Subscribe to stream data
@client.on('/sensors/accelerometer')
async def on_accel(value):
    update_visualization(value)
```

Subscriptions support `maxRate` (maximum updates per second) and `epsilon` (minimum change threshold) to downsample high-rate streams on the router side.

When to use streams: sensor data, motion capture, audio levels, video frames, any data over ~10 Hz.

## Gesture

A gesture models phased user input with four phases: `start`, `move`, `end`, and `cancel`. The router tracks only the current phase -- not the full history. Each gesture is identified by a numeric ID so multiple simultaneous gestures (e.g., multi-touch) can be tracked independently.

**JavaScript:**

```javascript
// Track a touch gesture
canvas.addEventListener('pointerdown', (e) => {
  client.gesture('/input/touch', e.pointerId, 'start', {
    x: e.clientX, y: e.clientY
  });
});

canvas.addEventListener('pointermove', (e) => {
  client.gesture('/input/touch', e.pointerId, 'move', {
    x: e.clientX, y: e.clientY
  });
});

canvas.addEventListener('pointerup', (e) => {
  client.gesture('/input/touch', e.pointerId, 'end', {
    x: e.clientX, y: e.clientY
  });
});
```

Gesture phases defined in the protocol:

| Phase | Meaning |
|-------|---------|
| `start` | Input began (finger down, pen contact) |
| `move` | Input position changed |
| `end` | Input completed normally (finger up) |
| `cancel` | Input interrupted (system event, out of bounds) |

### Gesture Deep Dive

The phased model maps directly to how physical input devices work -- a pen touches a tablet (`start`), moves across the surface (`move` x many), then lifts off (`end`). This lifecycle applies to touch, pen/stylus, mouse drag, MIDI CC sweeps, and any interaction that has a beginning, middle, and end.

**Pressure and metadata**: Gesture payloads can include arbitrary metadata alongside position. Pressure-sensitive pens pass `pressure` (0.0-1.0) and `tiltX`/`tiltY` values:

```javascript
await publisher.gesture('/touch/pen', {
  id: penId,
  phase: 'start',
  x: 50,
  y: 50,
  pressure: 0.1,
  tiltX: 0,
  tiltY: 45,
  metadata: {
    device: 'wacom-intuos',
    tool: 'brush',
    size: 24,
    color: '#ff5500'
  }
});
```

**Multi-touch**: Track multiple simultaneous gestures by giving each a unique ID. For a pinch-zoom, two fingers each get their own gesture stream:

```javascript
// Two fingers start simultaneously
client.gesture('/touch/finger/1', { id: finger1Id, phase: 'start', x: 200, y: 200 });
client.gesture('/touch/finger/2', { id: finger2Id, phase: 'start', x: 220, y: 200 });

// Fingers spread apart
for (let i = 0; i < 20; i++) {
  const spread = i * 5;
  client.gesture('/touch/finger/1', { id: finger1Id, phase: 'move', x: 200 - spread, y: 200 });
  client.gesture('/touch/finger/2', { id: finger2Id, phase: 'move', x: 220 + spread, y: 200 });
}

// Both fingers lift
client.gesture('/touch/finger/1', { id: finger1Id, phase: 'end', x: 100, y: 200 });
client.gesture('/touch/finger/2', { id: finger2Id, phase: 'end', x: 320, y: 200 });
```

**Coalescing**: When gesture `move` events arrive faster than the network can deliver them (e.g., a 240Hz pen tablet), the router coalesces rapid updates. Only the most recent position is forwarded to subscribers, reducing bandwidth while preserving responsiveness. In testing, 240Hz input is typically reduced to ~24 delivered updates per second -- a 90% bandwidth reduction with no perceptible latency increase.

**Receiving gestures**: Subscribe with `onGesture` (or `on` with a gesture address pattern) to receive the full lifecycle:

```javascript
subscriber.onGesture('/touch/**', (gesture) => {
  const { id, phase, x, y, pressure } = gesture;
  switch (phase) {
    case 'start':
      beginStroke(id, x, y);
      break;
    case 'move':
      extendStroke(id, x, y, pressure);
      break;
    case 'end':
      finishStroke(id);
      break;
    case 'cancel':
      discardStroke(id);
      break;
  }
});
```

## Timeline

A timeline carries keyframe automation data -- a sequence of timed values with easing curves. Timelines use Commit QoS (exactly-once, ordered) because they represent precise automation that must not be duplicated or reordered.

Timelines are immutable once published. To modify automation, publish a new timeline to the same address.

**JavaScript:**

```javascript
// Create a 3-second fade-in
client.timeline('/lights/stage/brightness', [
  { time: 0, value: 0.0, easing: 'linear' },
  { time: 1_000_000, value: 0.3, easing: 'ease-in' },
  { time: 3_000_000, value: 1.0, easing: 'ease-out' },
]);

// Schedule with start time and looping
client.timeline('/lights/stage/color', [
  { time: 0, value: [255, 0, 0], easing: 'ease-in-out' },
  { time: 2_000_000, value: [0, 0, 255], easing: 'ease-in-out' },
  { time: 4_000_000, value: [255, 0, 0], easing: 'ease-in-out' },
], { loop: true, startTime: client.time() + 1_000_000 });
```

Keyframe `time` values are offsets in microseconds from the timeline start. Available easing types:

| Easing | Behavior |
|--------|----------|
| `linear` | Constant speed (default) |
| `ease-in` | Slow start, fast end |
| `ease-out` | Fast start, slow end |
| `ease-in-out` | Slow start and end, fast middle |
| `step` | Instant change at next keyframe |
| `cubic-bezier` | Custom curve with 4 control points `[x1, y1, x2, y2]` |

The router interpolates between keyframes using the specified easing. Float and integer values interpolate numerically. Arrays of equal length interpolate element-wise. All other value types use step interpolation at the midpoint.

## QoS Levels

Every signal has a default Quality of Service level. You can override QoS per-message if needed.

| Level | Name | Delivery Guarantee | Use Case |
|-------|------|--------------------|----------|
| 0 | Fire | Best effort, no confirmation | High-rate data where drops are acceptable |
| 1 | Confirm | At least once delivery | State changes that must arrive |
| 2 | Commit | Exactly once, ordered | Transactions and automation that must not duplicate |

## Choosing a Signal Type

- **Need the current value later?** Use **Param**. Late joiners receive a snapshot. Conflict resolution applies.
- **One-shot trigger?** Use **Event**. No state stored. Delivered to current subscribers only.
- **High-rate continuous data?** Use **Stream**. Best-effort delivery, no storage, supports downsampling.
- **User input with phases?** Use **Gesture**. Tracks start/move/end/cancel lifecycle.
- **Timed automation?** Use **Timeline**. Keyframes with easing, exactly-once delivery.

## Bridge Translation

When CLASP bridges translate signals from other protocols, they map to the closest signal type:

| Source Protocol | Signal | CLASP Type |
|----------------|--------|------------|
| OSC message | Addressed value | Param |
| MIDI CC | Continuous controller | Param |
| MIDI Note On/Off | Trigger | Event |
| MQTT retained | Persistent topic | Param |
| MQTT non-retained | Transient topic | Event |
| Art-Net DMX | Channel value | Stream or Param |

## Next Steps

- [State Management](./state.md) -- how params track revisions, resolve conflicts, and sync to late joiners.
- [Addressing](./addressing.md) -- path hierarchy, wildcards, and namespace conventions.
- [Bundles](./bundles.md) -- grouping multiple signals into atomic or scheduled units.
