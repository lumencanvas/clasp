# clasp-state-defra

DefraDB-backed state store for CLASP routers with write-through caching.

## Architecture

```
Hot path (sub-100us):
  SET/GET --> DashMap (in-memory cache)
                |
                v (async, non-blocking)
  Background worker --> DefraDB (persistent)
                            |
                            v (Merkle CRDTs)
                        P2P sync to other nodes
```

The cache serves all reads and writes synchronously. A background worker flushes writes to DefraDB asynchronously. A separate sync worker polls DefraDB for changes from remote peers.

## Usage

```rust
use clasp_state_defra::{DefraStateStore, DefraStateConfig};
use clasp_core::Value;

let store = DefraStateStore::new("http://localhost:9181", DefraStateConfig::default()).await?;

// Start background workers
let _writer = store.start_writer();

// Hot path: sub-100us
store.set("/synth/osc1/freq", Value::Float(440.0), "session-1", None, false, false, None)?;
let freq = store.get("/synth/osc1/freq");  // Some(Float(440.0))

// Pattern matching
let all_osc = store.get_matching("/synth/osc*/**");
```

## Conflict resolution

CLASP conflict strategies (LWW, Max, Min, Lock, Merge) are enforced in the CLASP layer. DefraDB stores the resolved result. This means the in-memory cache is always authoritative for the local node, and DefraDB provides durable persistence and cross-node replication.

## Timestamps

CLASP uses microsecond timestamps internally. DefraDB stores timestamps as seconds (Int32 limit). Conversion is automatic and lossless at second granularity.

## License

MIT OR Apache-2.0
