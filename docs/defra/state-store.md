---
title: State Store
description: Write-through cache with DefraDB persistence and P2P sync
order: 3
---

# State Store

The `clasp-state-defra` crate provides a persistent state store that preserves CLASP's sub-100us signal routing performance.

## How It Works

```
SET /lights/brightness 0.8
         |
         v
    +---------+
    | DashMap  |  <-- hot path: sub-100us, synchronous
    | (cache)  |
    +---------+
         |
         | (async, non-blocking)
         v
    +---------+
    | DefraDB |  <-- persistence: async background worker
    | (store) |
    +---------+
         |
         | (Merkle CRDTs, automatic)
         v
    +---------+
    | DefraDB |  <-- replication: peer-to-peer sync
    | (peer)  |
    +---------+
```

1. `set()` updates the DashMap immediately (synchronous, sub-100us)
2. The write is queued to an unbounded channel (non-blocking)
3. A background worker flushes writes to DefraDB in batches
4. DefraDB replicates to peers via Merkle CRDTs
5. A sync worker polls DefraDB for remote changes and merges them into the cache

## Usage

```rust
use clasp_state_defra::{DefraStateStore, DefraStateConfig};
use clasp_core::Value;

let config = DefraStateConfig {
    max_cache_size: Some(10_000),
    sync_interval: Duration::from_secs(5),
    preload: true,           // load all state from DefraDB on startup
    ..Default::default()
};

let store = DefraStateStore::new("http://localhost:9181", config).await?;
let _writer = store.start_writer();

// Synchronous hot path
store.set("/synth/osc1/freq", Value::Float(440.0), "session-1", None, false, false, None)?;
let freq = store.get("/synth/osc1/freq"); // Some(Float(440.0))

// Pattern matching
let all_synth = store.get_matching("/synth/**");

// Flush pending writes
store.flush().await?;

// Cache statistics
let stats = store.cache_stats();
println!("cached: {}, pending: {}", stats.cached_params, stats.pending_writes);
```

## Configuration

| Field | Default | Description |
|-------|---------|-------------|
| `max_cache_size` | 10,000 | Maximum cached params (LRU eviction from cache, not DefraDB) |
| `sync_interval` | 5 seconds | How often to poll DefraDB for remote changes |
| `preload` | true | Load all state from DefraDB into cache on startup |
| `write_batch_size` | 100 | Number of writes to batch before flushing |

## Conflict Resolution

CLASP's conflict strategies (LWW, Max, Min, Lock, Merge) are enforced in the CLASP layer before writing to DefraDB. DefraDB stores the resolved result. When two peers write the same address concurrently:

1. Each peer resolves locally using its conflict strategy
2. Both write their result to DefraDB
3. DefraDB's LWW-Register CRDT picks a deterministic winner
4. The sync worker on the losing peer updates its cache with the winner

## Crash Recovery

With `preload: true`, the store loads all state from DefraDB when `DefraStateStore::new()` is called. No separate recovery step needed -- the constructor handles it.

## Timestamps

CLASP uses microsecond timestamps internally. DefraDB stores timestamps as seconds (Int32 limitation). Conversion is automatic and transparent. Sub-second precision is lost in DefraDB but preserved in the in-memory cache.
