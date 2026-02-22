---
title: "Handle Conflicts"
description: "Manage concurrent writes from multiple clients."
section: how-to
order: 2
---
# Handle Conflicts

Manage concurrent writes from multiple clients.

## Default: Last-Write-Wins

By default, the latest write wins (by timestamp):

```javascript
// Client A: set at t=100
// Client B: set at t=105
// Result: Client B's value wins
```

This works for most cases.

## Optimistic Locking

Check revision before writing:

```javascript
// Read current value and revision
const { value, revision } = await client.get('/config/setting', { meta: true });

// Modify value
const newValue = transform(value);

// Write with expected revision
try {
  await client.set('/config/setting', newValue, { revision });
} catch (error) {
  if (error.code === 'CONFLICT') {
    // Someone else changed it, retry
    console.log('Conflict, retrying...');
    await handleConflict();
  }
}
```

### Retry Pattern

```javascript
async function updateWithRetry(address, transform, maxRetries = 3) {
  for (let attempt = 0; attempt < maxRetries; attempt++) {
    const { value, revision } = await client.get(address, { meta: true });
    const newValue = transform(value);

    try {
      await client.set(address, newValue, { revision });
      return newValue;
    } catch (error) {
      if (error.code !== 'CONFLICT') throw error;
      // Retry on next iteration
    }
  }
  throw new Error('Max retries exceeded');
}

// Usage
await updateWithRetry('/counter', (v) => v + 1);
```

## Use Locks for Exclusive Access

When one user should control a value:

```javascript
await client.set('/mixer/fader/1', value, { lock: true });
// ... make changes ...
await client.set('/mixer/fader/1', value, { unlock: true });
```

See [Use Locks](use-locks.md).

## Use Bundles for Atomic Updates

When multiple values must change together:

```javascript
// All or nothing
await client.bundle([
  { set: ['/account/balance', 100] },
  { set: ['/account/last_transaction', Date.now()] }
]);
```

## Conflict Callbacks

Handle conflicts reactively:

```javascript
client.on('/path', (value, address, meta) => {
  if (meta.conflict) {
    console.log('Value was overwritten by:', meta.writer);
  }
});
```

## Avoid Conflicts by Design

Best approach: avoid conflicts structurally.

### Separate Namespaces

```javascript
// Instead of fighting over shared state
/volume = ???

// Give each client its own namespace
/client/alice/volume = 0.8
/client/bob/volume = 0.5

// Derive the final value
const volumes = await client.query('/client/*/volume');
const maxVolume = Math.max(...volumes.map(v => v.value));
```

### Use Events for Commands

```javascript
// Instead of setting state directly
await client.set('/lights/brightness', 0.8);

// Send a command, let one authority handle it
await client.emit('/commands/set-brightness', { value: 0.8 });

// Single handler processes commands
client.on('/commands/**', async (cmd, address) => {
  // This handler has exclusive control
  await client.set('/lights/brightness', cmd.value);
});
```

## Next Steps

- [Use Locks](use-locks.md)
- [Conflict Resolution Explanation](../../explanation/conflict-resolution.md)
