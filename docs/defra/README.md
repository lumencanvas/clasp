---
title: DefraDB Integration
description: Peer-to-peer persistent state via Merkle CRDTs
order: 7
---

# DefraDB Integration

CLASP integrates with [DefraDB](https://source.network/defradb), a peer-to-peer document database built on Merkle CRDTs. This gives CLASP durable, distributed state without requiring a central database: data syncs directly between peers.

## Why DefraDB?

CLASP is a real-time signal router. By default, state lives in memory and vanishes when the router restarts. DefraDB adds persistence and peer-to-peer replication while preserving CLASP's sub-100us hot path.

| Before DefraDB | After DefraDB |
|----------------|---------------|
| State lost on restart | State survives restarts (preloaded from DefraDB) |
| Single router only | Multi-router topologies (state syncs via CRDTs) |
| No historical queries | Full GraphQL query engine over signal history |
| File-based config | P2P config sync with version history |
| LibP2P-only peers | Browser, BLE, Serial peers via CLASP transports |

## Architecture

```
Client A ──> CLASP Router A ──> DefraDB Node 1
                                      |
                                 Merkle CRDT
                                   P2P Sync
                                      |
Client B ──> CLASP Router B ──> DefraDB Node 2
```

Each CLASP router connects to a local DefraDB instance. The hot path (SET/GET) stays in-memory via a DashMap cache. Writes flush asynchronously to DefraDB. DefraDB replicates documents to peers via Merkle CRDTs. A background sync worker pulls remote changes into the local cache.

## Getting Started

1. [Quick Start](quick-start.md): run DefraDB and connect a CLASP router
2. [Crate Guide](crates.md): what each crate does and when to use it
3. [State Store](state-store.md): the write-through cache architecture
4. [Configuration Sync](config-sync.md): P2P configuration management
5. [Bridge](bridge.md): real-time DefraDB change notifications
6. [Transport Tunnel](transport.md): DefraDB sync over CLASP transports
7. [Identity](identity.md): unified Ed25519 identity across systems
8. [Testing](testing.md): Docker setup, integration tests, E2E tests
