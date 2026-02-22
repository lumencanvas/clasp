---
title: "Use Locks"
description: "Request exclusive control of parameters."
section: how-to
order: 5
---
# Use Locks

Request exclusive control of parameters.

## When to Use Locks

Use locks when:
- One user should control a fader at a time
- Preventing race conditions during multi-step operations
- Implementing "take" functionality in show control

## Acquire a Lock

```javascript
// Request lock when setting
await client.set('/mixer/fader/1', 0.5, { lock: true });
```

If successful, you hold the lock. Others cannot write until you release it.

## Release a Lock

```javascript
// Release lock
await client.set('/mixer/fader/1', 0.5, { unlock: true });
```

Or release without changing value:

```javascript
await client.unlock('/mixer/fader/1');
```

## Handle Lock Denied

```javascript
try {
  await client.set('/mixer/fader/1', 0.5, { lock: true });
} catch (error) {
  if (error.code === 'LOCK_HELD') {
    console.log(`Lock held by: ${error.holder}`);
  }
}
```

## Lock Timeout

Locks automatically expire after 30 seconds of inactivity. Keep the lock alive by writing:

```javascript
// Writing extends the lock
setInterval(() => {
  client.set('/mixer/fader/1', currentValue, { lock: true });
}, 10000);  // Every 10 seconds
```

## Example: Fader Control

```javascript
let activeFader = null;

// On mouse down
async function onFaderStart(faderId) {
  try {
    await client.set(`/mixer/fader/${faderId}`, getValue(faderId), { lock: true });
    activeFader = faderId;
  } catch (error) {
    showMessage('Fader in use by another user');
  }
}

// On mouse move
function onFaderMove(faderId, value) {
  if (activeFader === faderId) {
    client.set(`/mixer/fader/${faderId}`, value, { lock: true });
  }
}

// On mouse up
async function onFaderEnd(faderId) {
  if (activeFader === faderId) {
    await client.set(`/mixer/fader/${faderId}`, getValue(faderId), { unlock: true });
    activeFader = null;
  }
}
```

## Check Lock Status

```javascript
const status = await client.get('/mixer/fader/1', { meta: true });
if (status.locked) {
  console.log(`Locked by: ${status.holder}`);
}
```

## Automatic Release on Disconnect

Locks are automatically released when the client disconnects. This prevents "orphaned" locks.

## Limitations

- Locks are per-address (not patterns)
- Lock timeout is server-configured
- High-frequency writes extend the lock

## Next Steps

- [Handle Conflicts](handle-conflicts.md)
- [Conflict Resolution Explanation](../../explanation/conflict-resolution.md)
