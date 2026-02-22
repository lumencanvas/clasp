---
title: Persistence
description: State snapshots, journal persistence, and REPLAY queries
order: 3
---

# Persistence

By default, CLASP state lives in memory and is lost when the relay restarts. Persistence lets you survive restarts and, optionally, query historical data. Two levels are available, from simple snapshots to a full append-only journal.

## Two Levels of Persistence

| Level | Flag | Feature Required | What It Does |
|-------|------|------------------|--------------|
| Snapshots | `--persist ./state.db` | None | Periodic state dumps, restored on restart |
| Journal | `--journal ./journal.db` | `--features journal` | Append-only event log, REPLAY queries, full history |

Start with snapshots if you just need state to survive restarts. Add the journal when you need historical queries, audit trails, or replay.

## State Snapshots

State snapshots are the simplest persistence option. The relay periodically dumps all param state to a SQLite database and restores it on startup.

```bash
clasp-relay --persist ./state.db --persist-interval 30
```

| Flag | Default | Description |
|------|---------|-------------|
| `--persist` | none | Path to SQLite database file (created if missing) |
| `--persist-interval` | `30` | Seconds between snapshot writes |

How it works:

1. On startup, if the database file exists, all stored param state is loaded into memory.
2. Every `persist-interval` seconds, the relay writes a full snapshot of current state to the database.
3. On graceful shutdown, a final snapshot is written.

Snapshots do not provide historical queries. Only the latest value of each param is stored. If the relay crashes between intervals, changes since the last snapshot are lost.

## Journal

The journal is an append-only event log. Every state change is recorded as a journal entry with a monotonic sequence number, timestamp, operation type, and data. The journal enables REPLAY queries and provides a complete audit trail.

### SQLite Journal

For durable persistence:

```bash
clasp-relay --journal ./journal.db
```

Every state change is appended to a SQLite database. The journal survives restarts and can grow to any size (subject to disk space).

For high-throughput deployments, use the batching variant which groups writes for better performance:

```bash
clasp-relay --journal ./journal.db --journal-batch-size 100 --journal-flush-ms 50
```

| Flag | Default | Description |
|------|---------|-------------|
| `--journal` | none | Path to SQLite journal database |
| `--journal-batch-size` | `100` | Max entries per batch write |
| `--journal-flush-ms` | `50` | Max milliseconds before flushing a partial batch |

### Memory Journal

For development and testing, an in-memory ring buffer journal is available:

```bash
clasp-relay --journal-memory
```

The memory journal does not survive restarts. It stores the most recent entries up to the ring buffer capacity. Use it when you want REPLAY queries during development without disk I/O.

## Memory vs SQLite

| | MemoryJournal | SqliteJournal | BatchingSqliteJournal |
|---|---|---|---|
| Durability | Lost on restart | Survives restart | Survives restart |
| Capacity | Limited by RAM | Limited by disk | Limited by disk |
| Write speed | Fastest | Moderate | Fast (batched) |
| REPLAY queries | Yes | Yes | Yes |
| Use case | Development | Production | High-throughput production |

## Journal Entries

Each journal entry contains:

| Field | Type | Description |
|-------|------|-------------|
| `sequence` | `u64` | Monotonically increasing sequence number |
| `timestamp` | `u64` | Microsecond Unix timestamp |
| `op` | `JournalOp` | Operation type (Set, Delete, Event, etc.) |
| `data` | bytes | Serialized operation data (address, value, metadata) |

Sequence numbers are gap-free within a single relay instance. After a restart, sequencing continues from the last stored value.

## REPLAY Queries

When the journal is enabled, clients can query historical data. A REPLAY request specifies a pattern, time range, and optional signal type filter.

Typical use cases:

- **Dashboards**: plot historical sensor values over time.
- **Debugging**: replay what happened in the last 5 minutes.
- **Audit trails**: review all writes to a security-sensitive path.

REPLAY returns journal entries matching the query, ordered by sequence number. Large result sets are paginated.

## Compaction

The SQLite journal supports compaction to prevent unbounded growth. Compaction removes entries before a given sequence number:

```bash
clasp-relay --journal ./journal.db --journal-compact-before 100000
```

You can also compact by age, removing entries older than a specified duration:

```bash
clasp-relay --journal ./journal.db --journal-compact-age 7d
```

After compaction, REPLAY queries can only return entries that remain. Run compaction during low-traffic periods to minimize impact.

## Combining Snapshots and Journal

You can use both persistence levels together:

```bash
clasp-relay --persist ./state.db --persist-interval 60 --journal ./journal.db
```

In this configuration, snapshots provide fast state restoration on startup (loading the latest values), while the journal provides historical queries and a complete audit trail. The relay restores from the snapshot first, then replays any journal entries written after the snapshot to bring state fully up to date.

## Next Steps

- [Rules Engine](./rules.md) -- server-side reactive automation
- [Federation](./federation.md) -- multi-site state sync
- [Production Checklist](../deployment/production-checklist.md) -- deployment considerations
