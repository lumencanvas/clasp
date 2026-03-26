---
title: Testing
description: Docker setup, integration tests, and E2E sync verification
order: 8
---

# Testing

## Docker Setup

The test infrastructure uses Docker Compose to run a 2-node DefraDB cluster with automatic P2P peering.

```bash
cd tests/defra

# Start nodes, provision 8 schemas, configure bidirectional replication
bash setup.sh

# Run all integration tests
cargo test -p clasp-journal-defra \
           -p clasp-registry-defra \
           -p clasp-config-defra \
           -p clasp-defra-transport \
           -p clasp-state-defra \
    -- --ignored --test-threads=1

# Run E2E sync tests
cargo run -p clasp-e2e --bin defra-sync-tests

# Tear down
bash teardown.sh
```

## What setup.sh Does

1. Starts two DefraDB containers (`clasp-defra-1` on port 9181, `clasp-defra-2` on port 9182)
2. Waits for both to be healthy
3. Provisions all 8 schemas on both nodes:
   - ClaspParam, ClaspJournalEntry, ClaspParamSnapshot
   - ClaspRouterConfig, ClaspConnectionConfig, ClaspBridgeConfig, ClaspRuleConfig, ClaspConfigSnapshot
4. Extracts each node's P2P multiaddr (Docker-network IP, not loopback)
5. Sets up bidirectional replication for all 8 collections

## Test Summary

| Crate | Unit | Integration | Notes |
|-------|------|-------------|-------|
| clasp-identity | 17 | 0 | No DefraDB needed |
| clasp-journal-defra | 20 | 4 | append, query, snapshot, compact |
| clasp-registry-defra | 6 | 6 | CRUD, find by tag/namespace |
| clasp-defra-bridge | 17 | 0 | Mock-based (no router needed) |
| clasp-config-defra | 13 | 5 | CRUD, snapshots, import/export |
| clasp-defra-transport | 10 | 2 | Peer registration, poll updates |
| clasp-state-defra | 19 | 3 | Write-through, load, P2P sync |

## E2E Sync Tests

The `defra-sync-tests` binary verifies end-to-end state propagation:

1. **test_journal_sync**: write journal entry on node 1, verify it appears on node 2
2. **test_state_store_sync**: set param via DefraStateStore on node 1, load from node 2
3. **test_config_sync**: save router config on node 1, query from node 2

All use unique UUIDs to avoid collisions. Tests skip gracefully if DefraDB is not running.

## Benchmarks

```bash
# Run cache performance benchmarks
cargo bench -p clasp-state-defra --bench cache_bench

# Quick smoke test (single iteration)
cargo bench -p clasp-state-defra --bench cache_bench -- --test
```

Benchmark groups:
- `cache_get`: hit and miss lookups
- `cache_set`: new inserts and updates
- `cache_pattern_match`: prefix filtering
- `concurrent_read_write`: interleaved operations
- `cache_scaling`: GET latency at 100, 1k, 10k, 100k entries

## CI

The `.github/workflows/defra-integration.yml` workflow runs on every push that touches DefraDB crate files. It starts two DefraDB service containers and runs all unit + integration tests.
