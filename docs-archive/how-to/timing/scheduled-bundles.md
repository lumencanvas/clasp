---
title: "Scheduled Bundles"
description: "Execute messages at a specific synchronized time."
section: how-to
order: 3
---
# Scheduled Bundles

Execute messages at a specific synchronized time.

## Schedule for Future

```javascript
// Execute in 100ms
await client.bundle([
  { set: ['/light/1/brightness', 1.0] },
  { set: ['/light/2/brightness', 0.0] }
], { at: client.time() + 100000 });  // 100,000 µs = 100ms
```

All devices execute at the same synchronized time.

## Schedule at Absolute Time

```javascript
// Execute at specific time (microseconds)
const targetTime = 1704067200000000;  // Specific moment
await client.bundle([...], { at: targetTime });
```

## Multiple Scheduled Events

Schedule a sequence:

```javascript
const now = client.time();

// Fade over 2 seconds (10 steps)
for (let i = 0; i <= 10; i++) {
  const t = now + (i * 200000);  // Every 200ms
  const brightness = 1.0 - (i / 10);

  await client.bundle([
    { set: ['/light/1/brightness', brightness] }
  ], { at: t });
}
```

## Execution Tolerance

Scheduled bundles execute within ±1ms of the specified time.

## Immediate Bundles

Without `at`, bundles execute immediately but atomically:

```javascript
// Execute now, but atomically
await client.bundle([
  { set: ['/light/1', 1.0] },
  { set: ['/light/2', 0.0] }
]);
```

## Cancel Scheduled Bundle

```javascript
const bundleId = await client.bundle([...], { at: futureTime });

// Cancel before execution
await client.cancelBundle(bundleId);
```

## Example: Synchronized Cue

```javascript
// All devices trigger at the same moment
const cueTime = client.time() + 5000000;  // 5 seconds from now

await client.bundle([
  { emit: ['/cue/lights', { scene: 'finale' }] },
  { emit: ['/cue/audio', { track: 'ending' }] },
  { emit: ['/cue/video', { clip: 'outro' }] }
], { at: cueTime });
```

## Next Steps

- [Atomic Bundles](bundle-atomic.md)
- [Clock Sync](clock-sync.md)
