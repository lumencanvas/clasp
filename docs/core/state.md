---
title: State Management
description: How CLASP tracks, resolves, and syncs parameter state
order: 2
---

# State Management

Every Param signal creates state that CLASP tracks with revisions, timestamps, and conflict resolution. Late-joining clients receive a snapshot of current state so they never miss the current value. This page covers how state is stored, read, updated, and evicted.

## How State Works

When a client sends a SET message, the router creates or updates a `ParamState` entry. Each entry tracks:

| Field | Type | Description |
|-------|------|-------------|
| `value` | any | Current value (float, int, string, bool, array, map, bytes, null) |
| `revision` | u64 | Monotonic counter, increments on every accepted write |
| `writer` | string | Session ID of the last client to write this param |
| `timestamp` | u64 | Microsecond timestamp of last write |
| `last_accessed` | u64 | Microsecond timestamp of last read or write (used for TTL eviction) |
| `strategy` | ConflictStrategy | How concurrent writes are resolved (default: LWW) |
| `lock_holder` | string or null | Session ID holding exclusive lock, if any |
| `meta` | object or null | Optional metadata: unit, range, default value |
| `origin` | string or null | Origin router ID (for federation loop prevention) |

Revisions start at 1 and increment by 1 on every accepted write. They never decrease. This gives every subscriber an unambiguous ordering of writes to a given address.

## Reading State

There are three ways to read param state:

**On-demand read** -- request the current value at any time:

```javascript
const value = await client.get('/lights/room1/brightness');
```

```python
value = await client.get('/lights/room1/brightness')
```

**Live subscription** -- receive updates as they happen:

```javascript
client.on('/lights/room1/brightness', (value, meta) => {
  console.log(`brightness: ${value} (rev ${meta.revision}, writer: ${meta.writer})`);
});
```

```python
@client.on('/lights/room1/brightness')
async def on_brightness(value, meta):
    print(f"brightness: {value} (rev {meta.revision})")
```

**Wildcard subscription** -- subscribe to multiple params at once:

```javascript
client.on('/lights/*/brightness', (value, meta) => {
  console.log(`${meta.address}: ${value}`);
});
```

## Late-Joiner Sync

When a client subscribes to a pattern, the router immediately sends a SNAPSHOT message containing the current value, revision, writer, and timestamp for every matching param. This means new clients start with the correct state without any extra coordination.

The snapshot contains a `ParamValue` for each matching address:

```
SNAPSHOT {
  params: [
    { address: "/lights/room1/brightness", value: 0.75, revision: 12, writer: "session-abc", timestamp: 1708531200000000 },
    { address: "/lights/room2/brightness", value: 0.50, revision: 3, writer: "session-def", timestamp: 1708531190000000 },
  ]
}
```

After the snapshot, the client receives live updates for any further writes.

## Conflict Resolution

When multiple clients write to the same param concurrently, the router applies a conflict resolution strategy. The strategy is set per-param and defaults to LWW.

| Strategy | Behavior | Use Case |
|----------|----------|----------|
| `lww` | Last Write Wins -- highest timestamp wins (default) | General-purpose params |
| `max` | Keep the maximum numeric value, reject lower | Bidding, high-score |
| `min` | Keep the minimum numeric value, reject higher | Auction floor, threshold |
| `lock` | Only the lock holder can write | Exclusive device control |
| `merge` | Accept all writes, application handles merge | Collaborative editing |

For `max` and `min`, non-numeric values fall back to LWW behavior.

**Optimistic concurrency** -- you can include an expected revision in a SET to detect conflicts:

```javascript
// Only update if the param is still at revision 5
client.set('/lights/room1/brightness', 0.8, { revision: 5 });
// If another write happened first, the router returns a RevisionConflict error
```

## Locks

Locks provide exclusive write access to a param. When a lock is held, only the lock holder can update the value. Other writers receive a `LockHeld` error.

```javascript
// Acquire lock
client.set('/lights/room1/brightness', 0.5, { lock: true });

// Only this client can write now
client.set('/lights/room1/brightness', 0.8);

// Release lock
client.set('/lights/room1/brightness', 0.8, { unlock: true });
```

```python
# Acquire lock
await client.set('/lights/room1/brightness', 0.5, lock=True)

# Release lock
await client.set('/lights/room1/brightness', 0.8, unlock=True)
```

Use locks when a single controller needs exclusive access to a device -- for example, one lighting console controlling a fixture without interference from other clients.

## TTL and Eviction

Params expire after a configurable Time-To-Live (TTL) based on their `last_accessed` timestamp. The default TTL is 1 hour (3600 seconds).

**Router configuration:**

| Flag | Description | Default |
|------|-------------|---------|
| `--param-ttl <seconds>` | Set param TTL | 3600 (1 hour) |
| `--no-ttl` | Disable TTL expiration entirely | - |
| `--max-params <count>` | Maximum number of params in the store | 10,000 |

When the store reaches `max-params`, the router evicts entries based on the configured strategy:

| Strategy | Behavior |
|----------|----------|
| LRU (default) | Evict the least recently accessed param |
| OldestFirst | Evict the param with the oldest creation timestamp |
| RejectNew | Reject new params, return an `AtCapacity` error |

Updating or reading an existing param always succeeds regardless of capacity -- eviction only applies when creating new params.

## Persistence

By default, state is held in memory and lost on router restart. For durable state, enable persistence:

| Flag | Description |
|------|-------------|
| `--persist ./state.db` | Save param snapshots to disk |
| `--journal ./journal.db` | Write a full event journal for replay |

With journaling enabled, clients can request a REPLAY of past events for a pattern within a time range:

```javascript
// Replay last 5 minutes of brightness changes
const events = await client.replay('/lights/*/brightness', {
  from: client.time() - 5 * 60 * 1_000_000,
});
```

## Next Steps

- [Addressing](./addressing.md) -- path hierarchy and wildcard patterns for subscriptions.
- [Bundles](./bundles.md) -- grouping multiple state changes into atomic operations.
- [Signal Types](./signals.md) -- the 5 signal types and their relationship to state.
