# clasp-journal

Append-only event journal for crash recovery and state persistence in CLASP.

## Features

- **Append-Only Log** - Immutable, ordered record of SET and PUBLISH operations
- **Pattern Queries** - Retrieve entries matching CLASP address patterns
- **Time Range Queries** - Filter by timestamp with microsecond precision
- **Snapshots** - Save and restore full state snapshots for fast recovery
- **Compaction** - Remove old entries to reclaim storage
- **Pluggable Storage** - In-memory ring buffer or SQLite backends
- **Router Integration** - Wire into the router with `Router::with_journal()`

## Installation

```toml
[dependencies]
clasp-journal = "3.5"

# With SQLite persistence
clasp-journal = { version = "3.5", features = ["sqlite"] }
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `sqlite` | Enables `SqliteJournal` for persistent storage |

## Usage

### In-Memory Journal

```rust
use clasp_journal::{MemoryJournal, Journal, JournalEntry};
use clasp_core::Value;

// Create with capacity (evicts oldest entries when full)
let journal = MemoryJournal::new(10_000);

// Or use default capacity (10,000 entries)
let journal = MemoryJournal::default_capacity();

// Append a SET entry
let entry = JournalEntry::from_set(
    "/lights/room1/brightness".to_string(),
    Value::Float(0.75),
    1,                          // revision
    "session-abc".to_string(),  // author
    1708500000000000,           // timestamp (microseconds)
);
let seq = journal.append(entry).await?;
```

### SQLite Journal

```rust
#[cfg(feature = "sqlite")]
use clasp_journal::SqliteJournal;

let journal = SqliteJournal::new("journal.db")?;
// Same Journal API as MemoryJournal
```

### Query Entries

```rust
use clasp_core::SignalType;

// Query by pattern and time range
let entries = journal.query(
    "/lights/**",               // address pattern
    Some(1708400000000000),     // from timestamp (microseconds)
    Some(1708500000000000),     // to timestamp
    Some(100),                  // limit
    &[SignalType::Param],       // signal types
).await?;

// Get entries since a sequence number
let recent = journal.since(seq, Some(50)).await?;

// Get latest sequence number
let latest = journal.latest_seq().await?;
```

### Snapshots

```rust
use clasp_journal::ParamSnapshot;

// Save a state snapshot
let snapshots = vec![
    ParamSnapshot {
        address: "/lights/room1/brightness".to_string(),
        value: Value::Float(0.75),
        revision: 1,
        writer: "session-abc".to_string(),
        timestamp: 1708500000000000,
    },
];
let snap_seq = journal.snapshot(&snapshots).await?;

// Load the most recent snapshot
if let Some(state) = journal.load_snapshot().await? {
    for param in state {
        println!("{} = {:?} (rev {})", param.address, param.value, param.revision);
    }
}
```

### Compaction

```rust
// Remove entries older than a given sequence number
let removed = journal.compact(1000).await?;
println!("Removed {} old entries", removed);
```

### Router Integration

```rust
use clasp_router::{Router, RouterConfig};
use clasp_journal::SqliteJournal;
use std::sync::Arc;

let journal = Arc::new(SqliteJournal::new("state.db")?);

let router = Router::new(RouterConfig::default())
    .with_journal(journal); // requires `journal` feature on clasp-router
```

The router automatically records all SET and PUBLISH operations to the journal and supports replay for crash recovery.

## Configuration Reference

### JournalEntry

| Field | Type | Description |
|-------|------|-------------|
| `seq` | `u64` | Monotonic sequence number (assigned by journal) |
| `timestamp` | `u64` | Wall clock timestamp (microseconds since epoch) |
| `author` | `String` | Entity or session ID of the author |
| `address` | `String` | CLASP address the entry applies to |
| `signal_type` | `SignalType` | `Param`, `Event`, `Stream`, `Gesture`, or `Timeline` |
| `value` | `Value` | The value that was set or published |
| `revision` | `Option<u64>` | Param revision (`Some` for SET, `None` for PUBLISH) |
| `msg_type` | `u8` | Message type code (`0x21` = SET, `0x20` = PUBLISH) |

### ParamSnapshot

| Field | Type | Description |
|-------|------|-------------|
| `address` | `String` | CLASP address |
| `value` | `Value` | Current param value |
| `revision` | `u64` | Current revision number |
| `writer` | `String` | Last writer ID |
| `timestamp` | `u64` | Timestamp (microseconds since epoch) |

### Journal Trait Methods

| Method | Description |
|--------|-------------|
| `append(entry)` | Append an entry, returns sequence number |
| `query(pattern, from, to, limit, types)` | Query by pattern, time, and signal type |
| `since(seq, limit)` | Get entries after a sequence number |
| `latest_seq()` | Get the highest sequence number |
| `snapshot(state)` | Save a state snapshot |
| `load_snapshot()` | Load the most recent snapshot |
| `compact(before_seq)` | Remove entries before a sequence number |
| `len()` | Total number of entries |

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

---

Maintained by [LumenCanvas](https://lumencanvas.studio)
