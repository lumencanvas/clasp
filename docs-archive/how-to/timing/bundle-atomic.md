---
title: "Atomic Bundles"
description: "Execute multiple messages as a single atomic operation."
section: how-to
order: 1
---
# Atomic Bundles

Execute multiple messages as a single atomic operation.

## Why Bundles?

Without bundles, separate messages may arrive at different times:

```javascript
// These may execute with gaps
await client.set('/light/1', 1.0);
await client.set('/light/2', 0.0);  // Brief moment where both are at old values
```

With bundles, all messages execute together:

```javascript
// These execute atomically
await client.bundle([
  { set: ['/light/1', 1.0] },
  { set: ['/light/2', 0.0] }
]);
```

## Basic Usage

```javascript
await client.bundle([
  { set: ['/path1', value1] },
  { set: ['/path2', value2] },
  { emit: ['/event', payload] }
]);
```

## Bundle Types

Mix different operations:

```javascript
await client.bundle([
  { set: ['/lights/brightness', 0.8] },     // Param
  { emit: ['/cue/fire', { id: 1 }] },       // Event
  { set: ['/status/updated', Date.now()] }  // Another param
]);
```

## Error Handling

If any operation fails, no changes are applied:

```javascript
try {
  await client.bundle([
    { set: ['/valid/path', 1] },
    { set: ['/invalid/path', 2] }  // This fails
  ]);
} catch (error) {
  // Neither value was changed
}
```

## Large Bundles

Bundles can contain many operations:

```javascript
// Update all DMX channels atomically
const ops = [];
for (let ch = 1; ch <= 512; ch++) {
  ops.push({ set: [`/dmx/0/${ch}`, values[ch]] });
}
await client.bundle(ops);
```

## Scheduled Atomic Bundles

Combine atomicity with scheduling:

```javascript
await client.bundle([
  { set: ['/light/1', 1.0] },
  { set: ['/light/2', 0.0] }
], { at: client.time() + 100000 });
```

## Next Steps

- [Scheduled Bundles](scheduled-bundles.md)
- [Timing Model Explanation](../../explanation/timing-model.md)
