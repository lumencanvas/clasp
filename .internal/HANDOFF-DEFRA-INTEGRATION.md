# CLASP + DefraDB Integration Handoff

Last updated: 2026-03-26

## Status: ALL 7 PHASES COMPLETE + FULLY TESTED + PUBLISHED + DOCUMENTED

**108 unit tests + 20/20 integration tests pass (including P2P two-store sync)**
**All 7 crates published to crates.io v4.2.0**
**CLI tool supports --journal --journal-backend defra --journal-defra-url**

| Phase | Crate | Status | Tests (pass/ignored) |
|-------|-------|--------|---------------------|
| 1. Unified Identity | `clasp-identity` | COMPLETE | 17/0 |
| 2. Journal Backend | `clasp-journal-defra` | COMPLETE | 21/4 |
| 3. Entity Store | `clasp-registry-defra` | COMPLETE | 7/6 |
| 4. Change Bridge | `clasp-defra-bridge` | COMPLETE | 19/0 |
| 5. Config Persistence | `clasp-config-defra` | COMPLETE | 14/5 |
| 6. Browser Transport | `clasp-defra-transport` | COMPLETE | 11/2 |
| 7. State Store | `clasp-state-defra` | COMPLETE | 20/3 |
| Test Infrastructure | — | COMPLETE | — |
| **Total** | **7 new crates** | — | **108/20** |

## New Crates (7)

| Crate | Purpose | Key Trait/Interface |
|-------|---------|-------------------|
| `clasp-identity` | One Ed25519 key -> EntityId + DID + PeerID | `Identity` struct |
| `clasp-journal-defra` | Journal trait over DefraDB GraphQL API | `impl Journal for DefraJournal` |
| `clasp-registry-defra` | EntityStore trait over DefraDB | `impl EntityStore for DefraEntityStore` |
| `clasp-defra-bridge` | Bidirectional DefraDB ↔ CLASP signals | `DefraBridge` + `SignalSender`/`SignalReceiver` traits |
| `clasp-config-defra` | Config CRUD + snapshots + JSON import/export | `DefraConfigStore` |
| `clasp-defra-transport` | Tunnel DefraDB sync over CLASP transports | `DefraTunnel` + `TunnelMessage` protocol |
| `clasp-state-defra` | Write-through cache + async DefraDB persistence | `DefraStateStore` with DashMap + background writer |

## Architecture Decision Records

### ADR-001: HTTP/GraphQL for DefraDB communication
- DefraDB is Go, CLASP is Rust. Chose HTTP/GraphQL: cleanest boundary, no build complexity
- Latency acceptable: journal.append() is async fire-and-forget, not on hot path
- DefraDB WASM/C bindings exist but maturity unclear

### ADR-002: Sequence numbers are local, not global
- Local AtomicU64 counter, initialized from max(seq) on startup
- Cross-router sync uses DefraDB's native Merkle CRDT replication, not seq-based

### ADR-003: Pattern matching simplification
- CLASP `*`/`**` both map to DefraDB `%` (no path-segment awareness in _like)
- Acceptable for journal queries; exact matching happens at CLASP router level

### ADR-004: No cyclic re-export from clasp-journal
- `clasp-journal-defra` is standalone; consumers depend on it directly
- Avoids cyclic dependency that `defra` feature on `clasp-journal` would create

### ADR-005: Bridge uses trait injection
- `SignalSender` and `SignalReceiver` traits, not direct `clasp-client` dependency
- Enables mock-based unit testing without a running router
- `OriginTracker` with TTL-based suppression prevents echo loops

### ADR-006: Write-through cache for state store
- DashMap in-memory cache for sub-100us hot path reads/writes
- Unbounded channel + background worker for async DefraDB writes
- Background sync worker polls DefraDB to catch remote peer changes
- CLASP conflict strategies (Max/Min/Lock) enforced in CLASP layer; DefraDB stores results

## Files Modified (existing)

- `Cargo.toml` — 7 new workspace members, deps: reqwest, chrono, serde_bytes
- `crates/clasp-journal/src/lib.rs` — doc comment update

## Test Infrastructure

- `tests/defra/docker-compose.yml` — 2-node DefraDB cluster with health checks
- `tests/defra/setup.sh` / `teardown.sh` — start/stop scripts
- `.github/workflows/defra-integration.yml` — CI pipeline (unit + integration jobs)

## Research

- `.internal/RESEARCH-DEFRA-SOURCE-NETWORK.md` — full comparison and integration analysis
- `.claude/plans/imperative-growing-wreath.md` — detailed phased plan with E2E testing strategy

## Blockers and Risks

1. **DefraDB v1.0.0-rc1 stability**: RC, not GA. API may change before 1.0
2. **Docker image availability**: Verify `sourcenetwork/defradb:v1.0.0-rc1` on Docker Hub
3. **P2P peering setup**: Node-to-node peering requires peer ID exchange at runtime
4. **Bulk delete**: DefraDB compact() uses query-then-delete (potential perf issue at scale)
5. **DefraDB array filtering**: `find_by_tag`/`find_by_namespace` may need client-side filtering if `_any` operator isn't available

## Completed Milestones

### 2026-03-26: All 7 phases complete
- Phases 1-2 built first (no dependencies), then 3-4 in parallel, then 5-7 in parallel
- All crates compile and pass tests in workspace context
- 108 unit tests pass, 0 fail

### 2026-03-26: Integration tested + published
- DefraDB `latest` image pulled and running via Docker Compose
- Fixed API compatibility: schema endpoint, mutation names (create_ -> add_), health check, Int32 timestamps
- Fixed test idempotency: unique IDs per run (UUID/nanos) to avoid stale data collisions
- **19/20 integration tests pass** against live DefraDB (2-node cluster)
- 1 test (`test_two_store_sync`) needs P2P peering setup between nodes — documented, not blocking
- All 7 crates published to crates.io as v4.2.0

### DefraDB API fixes applied
- Schema: `POST /api/v0/collections` with `Content-Type: text/plain` SDL body
- Mutations: `add_X` not `create_X`; `delete_X(docID: "...")` not `delete_X(docIDs: [...])`
- `upsert_X` syntax: `upsert_X(filter: {...}, add: {...}, update: {...})`
- `@index` directive works as expected
- Int fields are 32-bit — timestamps stored as seconds (converted from/to CLASP microseconds)
- `json_to_graphql_input()` utility converts JSON objects to GraphQL input syntax (unquoted keys)

### 2026-03-26: Docs, CLI journal, P2P peering
- README.md added for all 7 crates
- Doc comments fixed (IdentityError, CacheStats, DefraStateError, peer_id functions)
- `clasp-router` CLI: added `--journal`, `--journal-backend`, `--journal-path`, `--journal-defra-url`
- Feature flags: `journal`, `journal-sqlite`, `journal-defra` on `clasp-router-server`
- `setup.sh`: fixed health check, fixed peer address extraction, bidirectional replication working
- **20/20 integration tests now pass** including `test_two_store_sync` (P2P state convergence verified)

## CLI Usage

```bash
# Memory journal (default, no persistence)
clasp-router --journal --journal-backend memory

# SQLite journal (persistent, single node)
clasp-router --journal --journal-backend sqlite --journal-path ./journal.db

# DefraDB journal (persistent + P2P sync)
clasp-router --journal --journal-backend defra --journal-defra-url http://localhost:9181
```

### 2026-03-26: Bridge app IPC, E2E tests, benchmarks
- Bridge app: `defra-health-check`, `defra-config-export`, `defra-config-import` IPC handlers
- Bridge app: server-manager passes `--journal-backend` and `--journal-defra-url` to router
- E2E test binary: `defra-sync-tests` — verifies journal, state, and config sync across 2 DefraDB nodes
- Criterion benchmarks: `cache_bench` for DashMap hot-path (get, set, pattern match, concurrent, scaling)
- Fix: all timestamps converted to seconds for DefraDB Int32 (journal, state store, queries)
- setup.sh: provisions all 8 schemas, replicates all 8 collections bidirectionally

## Next Steps

1. **Bridge app UI**: add DefraDB settings panel (URL, health indicator, sync status)
2. **Production hardening**: connection retry/backoff for DefraDB client, circuit breaker
3. **Contact Source Network**: discuss integration, potential upstream contributions
