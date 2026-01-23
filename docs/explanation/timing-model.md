# Timing Model

CLASP provides deterministic timing through clock synchronization and scheduled execution.

## Why Timing Matters

Creative applications need precise timing:

- **Music:** Triggers on beat
- **Lighting:** Synchronized fades
- **Video:** Frame-accurate cues
- **Installations:** Coordinated effects

Without synchronized clocks, each device is on its own timeline.

## Time Representation

CLASP uses 64-bit unsigned integers representing microseconds:

```javascript
const timestamp = 1704067200000000;  // Microseconds
//                └──────────────┘
//                About 54,000 years range
```

### Session Time vs Unix Time

| Type | Reference | Use Case |
|------|-----------|----------|
| **Session time** | Start of session | Live performance, relative timing |
| **Unix time** | 1970-01-01 | Logging, absolute scheduling |

Session time avoids timezone issues and provides relative timing.

## Clock Synchronization

### The Problem

Each device has its own clock:

```
Device A clock: 12:00:00.000
Device B clock: 12:00:00.050  (50ms ahead)
Device C clock: 11:59:59.980  (20ms behind)
```

Without sync, "execute at 12:00:01" means different things.

### SYNC Protocol

CLASP uses an NTP-like algorithm:

```
Client                              Router
  │                                    │
  │── SYNC { t1: 1000000 } ───────────►│  (Client sends at t1)
  │                                    │  (Router receives at t2)
  │◄── SYNC { t1, t2, t3 } ────────────│  (Router sends at t3)
  │                                    │
  │  (Client receives at t4)           │
```

Calculate offset:
```
roundTrip = (t4 - t1) - (t3 - t2)
offset = ((t2 - t1) + (t3 - t4)) / 2
```

### Sync Quality

After synchronization:

| Network | Typical Accuracy |
|---------|------------------|
| Wired LAN | ±1ms |
| WiFi | ±5-10ms |
| Internet | ±20-50ms |

Clients should sync every 30 seconds to maintain accuracy.

### API Usage

```javascript
// Get synchronized time
const now = client.time();

// Schedule 100ms in the future
const future = client.time() + 100000;
```

## Scheduled Bundles

Execute messages at a specific time:

```javascript
// Schedule a bundle for future execution
await client.bundle([
  { set: ['/light/1/brightness', 1.0] },
  { set: ['/light/2/brightness', 0.0] }
], { at: client.time() + 100000 });  // 100ms from now
```

### How It Works

1. Client sends BUNDLE with timestamp
2. Router receives and stores bundle
3. At scheduled time, router executes all messages
4. All subscribers receive updates simultaneously

### Execution Tolerance

Routers execute bundles within ±1ms of scheduled time on typical hardware.

### Use Cases

- **Lighting cues:** Coordinated fades
- **Music sync:** Triggers on beat
- **Show control:** Timed sequences
- **Animation:** Keyframe playback

## Timestamps on Messages

Individual messages can have timestamps:

```javascript
{
  type: "SET",
  address: "/sensor/temperature",
  value: 23.5,
  timestamp: 1704067200123456  // When the reading was taken
}
```

This records WHEN something happened, not when to execute.

## Jitter Handling

### The Problem

Network jitter causes timing variations:

```
Sent at:      0ms   16ms   32ms   48ms
Received at:  5ms   22ms   35ms   58ms  (jittery)
```

### Jitter Buffer

For high-rate streams, receivers can buffer:

```javascript
client.on('/sensor/motion', callback, {
  jitterBuffer: 50  // 50ms buffer
});
```

This smooths playback at the cost of latency.

### When to Use

| Data Type | Jitter Buffer? |
|-----------|----------------|
| Parameters | No (state matters) |
| Events | No (timing matters) |
| Streams over WiFi | Yes (smoothing helps) |
| Streams over LAN | Usually no |

## Timing Guarantees

CLASP provides **soft realtime** guarantees:

| Guarantee | Typical | Notes |
|-----------|---------|-------|
| Clock sync accuracy | ±1-5ms | LAN, after sync |
| Bundle execution | ±1ms | From scheduled time |
| Message latency | <1ms | LAN, single hop |

### What CLASP is NOT for

- Hard realtime (safety-critical)
- Sub-millisecond accuracy
- Industrial control systems
- Audio sample-accurate sync

For audio sync, use dedicated audio protocols and sync CLASP to the audio clock.

## Practical Examples

### Synchronized Light Fade

```javascript
const startTime = client.time() + 100000;  // Start in 100ms
const duration = 2000000;  // 2 seconds

// Schedule keyframes
for (let i = 0; i <= 10; i++) {
  const t = startTime + (duration * i / 10);
  const brightness = 1.0 - (i / 10);

  await client.bundle([
    { set: ['/light/1/brightness', brightness] },
    { set: ['/light/2/brightness', brightness] }
  ], { at: t });
}
```

### Beat-Synced Triggers

```javascript
// Assume BPM is known
const bpm = 120;
const beatDuration = 60000000 / bpm;  // Microseconds per beat
const nextBeat = calculateNextBeat(client.time(), bpm);

// Trigger on next beat
await client.bundle([
  { emit: ['/cue/flash', {}] }
], { at: nextBeat });

// Trigger on every beat
setInterval(() => {
  const beat = calculateNextBeat(client.time(), bpm);
  client.bundle([
    { emit: ['/cue/strobe', {}] }
  ], { at: beat });
}, beatDuration / 1000);  // Check every beat
```

### Recording and Playback

```javascript
// Record with timestamps
const recording = [];
client.on('/controller/**', (value, address) => {
  recording.push({
    time: client.time(),
    address,
    value
  });
});

// Playback at original timing
const playbackStart = client.time();
const recordingStart = recording[0].time;

for (const event of recording) {
  const offset = event.time - recordingStart;
  await client.bundle([
    { set: [event.address, event.value] }
  ], { at: playbackStart + offset });
}
```

## Best Practices

### Do Sync Clocks

```javascript
// Initial sync happens on connect
// Periodic sync is automatic
```

### Do Use Bundles for Coordination

```javascript
// Good: Atomic, synchronized
await client.bundle([
  { set: ['/light/1', 1.0] },
  { set: ['/light/2', 0.0] }
]);

// Less good: Sequential, may have gaps
await client.set('/light/1', 1.0);
await client.set('/light/2', 0.0);
```

### Don't Rely on Message Order

```javascript
// Bad: Assumes order
client.set('/a', 1);
client.set('/b', 2);  // May arrive before /a

// Good: Use bundle for ordered execution
client.bundle([
  { set: ['/a', 1] },
  { set: ['/b', 2] }
]);
```

### Don't Schedule Too Far Ahead

```javascript
// Reasonable
client.bundle([...], { at: client.time() + 10000000 });  // 10 seconds

// Risky (clock drift over time)
client.bundle([...], { at: client.time() + 3600000000 });  // 1 hour
```

## See Also

- [Clock Sync How-To](../how-to/timing/clock-sync.md) — Configuration
- [Scheduled Bundles How-To](../how-to/timing/scheduled-bundles.md) — Usage
- [QoS Levels](../reference/protocol/qos.md) — Reliability and timing
