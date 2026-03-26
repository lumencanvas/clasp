# CLASP + DefraDB: P2P State Sync for Real-Time Applications

CLASP is a real-time signal router. DefraDB is a peer-to-peer document database. Together they form a complete edge-first stack: sub-100us signal routing with persistent, distributed, conflict-free storage.

CLASP handles the nervous system (routing signals between OSC surfaces, MIDI controllers, DMX fixtures, web dashboards, IoT sensors). DefraDB handles the memory (storing state as documents that replicate peer-to-peer via Merkle CRDTs without central coordination).

## The Six DefraDB Crates

### clasp-journal-defra

The foundation. Implements CLASP's `Journal` trait against DefraDB's HTTP/GraphQL API via `reqwest`. Handles schema provisioning (`POST /api/v0/collections` with SDL), GraphQL mutation building, retry with exponential backoff (3 retries, 100ms base), and connection timeouts (10s request, 5s connect).

Every other DefraDB crate depends on the `DefraClient` exported by this crate.

By using DefraDB as the journal backend, CLASP gets multi-node replication of journal state for free via Merkle CRDTs. No explicit federation protocol needed.

```rust
use clasp_journal_defra::DefraJournal;
use clasp_journal::Journal;

let journal = DefraJournal::connect("http://localhost:9181").await?;
journal.append(entry).await?;
let history = journal.query("/lights/**", None, None, Some(100), &[]).await?;
```

**Tests**: 20 unit (conversions, query format, doc parsing) + 4 integration (append, query, snapshot, compact against live DefraDB)

### clasp-state-defra

The most consequential crate. A write-through cache backed by DefraDB.

The hot path (`get`/`set`) operates on a DashMap in-memory cache at sub-100us. Writes queue to an unbounded channel and flush asynchronously to DefraDB via a background worker. A separate sync task polls DefraDB for remote changes from P2P peers and merges them into the local cache using revision comparison.

This is the crate that makes CLASP's ephemeral router state durable and distributed.

```rust
use clasp_state_defra::{DefraStateStore, DefraStateConfig};
use clasp_core::Value;

let store = DefraStateStore::new("http://localhost:9181", DefraStateConfig::default()).await?;
let _writer = store.start_writer();

// Hot path: sub-100us (DashMap, no network)
store.set("/synth/osc1/freq", Value::Float(440.0), "session-1", None, false, false, None)?;
let freq = store.get("/synth/osc1/freq"); // Some(Float(440.0))
```

Configuration knobs via `DefraStateConfig`:
- `max_cache_size`: default 10,000 params
- `sync_interval`: default 5 seconds
- `preload`: load all state from DefraDB on startup
- `write_batch_size`: batch size for async flush

**Tests**: 19 unit (cache, conflict resolution, TTL, conversions) + 3 integration (write-through persistence, load from DefraDB, two-node P2P sync)

### clasp-defra-bridge

Bidirectional signal bridge. Maps DefraDB documents to CLASP addresses:

```
/defra/{collection}/{docID}          -- whole document
/defra/{collection}/{docID}/{field}  -- specific field
```

A `DefraWatcher` polls DefraDB for document changes and emits CLASP signals. A `DefraWriter` subscribes to `/defra/**` on the CLASP router and writes incoming signals back to DefraDB. An `OriginTracker` prevents echo loops with a configurable TTL-based suppression window (default 5s).

This is what lets any CLASP client -- a TouchOSC surface, a web dashboard, an Arduino -- subscribe to database mutations as real-time signals.

```rust
use clasp_defra_bridge::{DefraBridge, SignalSender, SignalReceiver};

let bridge = DefraBridge::new("http://localhost:9181", vec!["SensorData".into()])
    .with_echo_ttl(Duration::from_secs(10));

bridge.run(sender, receiver, shutdown).await?;
```

Uses trait injection (`SignalSender`, `SignalReceiver`) instead of depending on `clasp-client` directly. Testable with mocks.

**Tests**: 17 unit (addresses, conversions, origin tracker, watcher diff detection, writer) + 2 doc-tests

### clasp-defra-transport

Tunnels DefraDB P2P sync over CLASP transports. DefraDB uses LibP2P, which has limited browser support. CLASP has production WebSocket, WebRTC, and WASM clients.

This crate wraps DefraDB sync messages as `TunnelMessage` variants:

| Message | Purpose |
|---------|---------|
| `SyncRequest` | Request sync for a collection |
| `DagBlock` | Transfer an IPLD DAG node |
| `HeadUpdate` | Notify of new document commit |
| `BlockAck` | Acknowledge received blocks |
| `PeerInfo` | Exchange peer capabilities |
| `QueryForward` | Forward GraphQL query to peer |
| `QueryResponse` | Return query results |

Address namespace: `/defra/sync/{peer_id}/{collection}`

This breaks DefraDB out of the LibP2P-only world. Browsers, BLE devices, serial-connected microcontrollers, anything behind restrictive NATs -- all can participate in DefraDB replication through CLASP's transport layer.

**Tests**: 10 unit (protocol roundtrips, address parsing, peer registration) + 2 integration

### clasp-registry-defra

Stores CLASP entity identity records (devices, users, services, routers) in DefraDB. Implements the `EntityStore` trait. Device registrations, status changes, and revocations replicate automatically across peers via Merkle CRDTs.

```rust
use clasp_registry_defra::DefraEntityStore;
use clasp_registry::EntityStore;

let store = DefraEntityStore::connect("http://localhost:9181").await?;
store.create(&entity).await?;
store.update_status(&entity_id, EntityStatus::Revoked).await?;
```

**Tests**: 6 unit (conversions, hex encoding) + 6 integration (CRUD, find by tag/namespace, status propagation)

### clasp-config-defra

Stores router, connection, bridge, and rule configurations in DefraDB. Enables:

- P2P config sync between team members via Merkle CRDTs
- Version history via DefraDB's Merkle DAG (time-travel queries)
- Snapshot management (capture/restore full config state)
- JSON import/export (compatible with file-based config)
- DID-based owner field for access control

```rust
use clasp_config_defra::{DefraConfigStore, RouterConfig};

let store = DefraConfigStore::new("http://localhost:9181").await?;
store.save_router(&config).await?;
let snapshot = store.export_json().await?;
```

**Tests**: 13 unit (type roundtrips, conversion parsing) + 5 integration (CRUD, snapshots, import/export)

## Supporting Crate: clasp-identity

One Ed25519 keypair produces three interoperable identity formats:

- **CLASP EntityId**: `clasp:<base58>` (compatible with clasp-registry)
- **W3C DID**: `did:key:z6Mk...` (Ed25519 multicodec + base58btc)
- **libp2p PeerID**: `12D3KooW...` (protobuf + identity multihash + base58btc)

No libp2p dependency. PeerID encoding implemented manually.

```rust
use clasp_identity::Identity;

let id = Identity::generate();
println!("{}", id.entity_id()); // clasp:3vQB7B...
println!("{}", id.did());       // did:key:z6Mk...
println!("{}", id.peer_id());   // 12D3KooW...
```

**Tests**: 17 unit (deterministic generation, format validation, roundtrips, cross-format key consistency, sign/verify)

## What This Unlocks

### State survives router restarts

`DefraStateStore` preloads all state from DefraDB on startup. Kill the router, restart it, every param is back at its last value. Late-joining clients get the full current state -- including everything written before the last reboot.

### Multi-router topologies without federation

Two CLASP routers backed by peered DefraDB nodes converge state automatically. Router A sets `/lights/brightness` to 0.8. DefraDB syncs the document. Router B's sync worker picks it up. Any client on Router B sees the change. No federation protocol needed -- DefraDB's Merkle CRDTs handle it.

### Browser-to-browser DefraDB sync

The transport tunnel breaks DefraDB out of LibP2P. A web app using CLASP's WASM client can participate in DefraDB replication via WebSocket or WebRTC. This is something DefraDB alone cannot do today.

### Database mutations as real-time signals

The bridge maps DefraDB changes to subscribable CLASP signals. A dashboard subscribes to `/defra/SensorData/*/temperature` and gets real-time updates whenever any sensor document changes. CLASP's wildcard pattern matching applies to database content.

### Distributed configuration management

Show configs, device registrations, routing rules -- all stored as DefraDB documents with automatic P2P sync, version history, and DID-based access control. A touring show connects at a new venue and pulls the full production config from any peer that has it.

## CLI Usage

```bash
# Memory journal (default, no persistence)
clasp-router --journal --journal-backend memory

# SQLite journal (persistent, single node)
clasp-router --journal --journal-backend sqlite --journal-path ./journal.db

# DefraDB journal (persistent + P2P sync)
clasp-router --journal --journal-backend defra --journal-defra-url http://localhost:9181
```

## Test Infrastructure

### Docker Compose (2-node DefraDB cluster)

```bash
cd tests/defra
bash setup.sh    # starts nodes, provisions 8 schemas, sets up bidirectional P2P replication
bash teardown.sh # stops and cleans up
```

### Test Summary

| Crate | Unit | Integration | Doc | Total |
|-------|------|-------------|-----|-------|
| clasp-identity | 17 | 0 | 0 | 17 |
| clasp-journal-defra | 20 | 4 | 1 | 25 |
| clasp-registry-defra | 6 | 6 | 1 | 13 |
| clasp-defra-bridge | 17 | 0 | 2 | 19 |
| clasp-config-defra | 13 | 5 | 1 | 19 |
| clasp-defra-transport | 10 | 2 | 0 | 12 |
| clasp-state-defra | 19 | 3 | 1 | 23 |
| **Total** | **102** | **20** | **6** | **128** |

Plus 3 E2E sync tests (`cargo run -p clasp-e2e --bin defra-sync-tests`) and 5 criterion benchmarks (`cargo bench -p clasp-state-defra`).

### E2E Sync Tests

Verify end-to-end state propagation across two DefraDB nodes:

1. `test_journal_sync` -- write journal entry on node 1, read from node 2 after P2P sync
2. `test_state_store_sync` -- set param via DefraStateStore on node 1, verify on node 2
3. `test_config_sync` -- save router config on node 1, load from node 2

All three pass with 3-5 second sync windows on a local Docker network.

## Architecture Decisions

### ADR-001: HTTP/GraphQL for DefraDB communication
DefraDB is Go, CLASP is Rust. HTTP/GraphQL is the cleanest boundary -- no CGO, no build complexity. Journal append is fire-and-forget async, so latency is acceptable.

### ADR-002: Sequence numbers are router-local
DefraDB documents get CIDs, not sequential IDs. Local AtomicU64 counter initialized from max(seq) on startup. Cross-router sync uses DefraDB's native Merkle CRDT replication.

### ADR-003: Pattern matching simplification
CLASP uses `*` (one segment) and `**` (multi-segment) wildcards. DefraDB `_like` only supports `%`. Both map to `%` for journal queries. Exact segment matching happens at the CLASP router level.

### ADR-004: No cyclic re-export
`clasp-journal-defra` is standalone, not a feature flag on `clasp-journal`. Avoids cyclic dependency.

### ADR-005: Bridge uses trait injection
`SignalSender` and `SignalReceiver` traits, not direct `clasp-client` dependency. Enables mock-based unit testing.

### ADR-006: Write-through cache
DashMap for sub-100us hot path. Unbounded channel + background worker for async DefraDB writes. CLASP conflict strategies (LWW, Max, Min, Lock) enforced in CLASP layer; DefraDB stores resolved results.

### ADR-007: Timestamps stored as seconds
DefraDB `Int` is 32-bit signed. CLASP timestamps (microseconds) are divided by 1,000,000 for storage and multiplied back on read. Valid until 2038. If DefraDB adds Int64, migrate to microsecond precision.

## DefraDB API Notes

Verified against DefraDB `latest` Docker image (2026-03):

- Schema provisioning: `POST /api/v0/collections` with raw SDL body (`Content-Type: text/plain`)
- Create mutation: `add_X` (not `create_X`)
- Delete mutation: `delete_X(docID: "...")` (singular, not `docIDs`)
- Upsert: `upsert_X(filter: {...}, add: {...}, update: {...})`
- Health check: `GET /health-check` returns `"Healthy"`
- `@index` directive works in SDL for secondary indexes
- Array fields (`[String]`) supported in schemas
- P2P replication: `defradb client p2p replicator add -c CollectionName <multiaddr>`
