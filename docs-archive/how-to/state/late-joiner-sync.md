---
title: "Late Joiner Sync"
description: "Synchronize state when clients connect after values are set."
section: how-to
order: 3
---
# Late Joiner Sync

Synchronize state when clients connect after values are set.

## How It Works

When a client subscribes, CLASP sends a SNAPSHOT of matching current values:

```javascript
client.on('/lights/**', (value, address) => {
  // First calls: current state of all /lights/** params
  // Subsequent calls: updates as they happen
});
```

This happens automatically. No special handling needed.

## Example

```javascript
// Assume /lights/1/brightness = 0.8 was set earlier

// New client connects
const client = await new ClaspBuilder(url).connect();

// Subscribe
client.on('/lights/**', (value, address) => {
  console.log(address, value);
});

// Output (immediately):
// /lights/1/brightness 0.8
```

## Skip Initial Snapshot

If you only want updates, not current state:

```javascript
client.on('/lights/**', callback, { skipInitial: true });
```

## Request History

Get recent values (if router tracks history):

```javascript
client.on('/sensor/temp', callback, { history: 10 });
// Receives last 10 values, then updates
```

## Sync After Reconnection

On reconnection, snapshots are re-sent:

```javascript
client.onReconnect(() => {
  // Subscriptions restored
  // Snapshots re-sent automatically
});
```

## Manual Sync

Force sync at any time:

```javascript
// Re-request current values
const state = await client.query('/lights/**');
for (const param of state) {
  handleUpdate(param.address, param.value);
}
```

## Events Are Not Synced

Events are ephemeral—late joiners don't receive past events:

```javascript
// Events emitted before subscribe are lost
await client.emit('/cue/fire', { id: 1 });

// Later subscriber won't receive the above event
client.on('/cue/*', callback);  // Only future events
```

Use Params for state that must persist.

## Next Steps

- [Subscribe to Changes](subscribe-to-changes.md)
- [State Management Explanation](../../explanation/state-management.md)
