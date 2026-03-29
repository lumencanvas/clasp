---
title: Crate Guide
description: Overview of the seven DefraDB integration crates
order: 2
---

# Crate Guide

The DefraDB integration consists of seven crates. Six handle specific integration concerns; one provides unified identity.

## Dependency Graph

```
clasp-core
    |
clasp-journal ──> clasp-journal-defra (DefraClient, DefraJournal)
    |                      |
    |          +-----------+------------+-------------+-------------+
    |          |           |            |             |             |
    |    clasp-registry  clasp-defra  clasp-config  clasp-state  clasp-defra
    |      -defra        -bridge      -defra        -defra       -transport
    |
clasp-identity (standalone, no DefraDB dependency)
```

## When to Use Each Crate

| You want to... | Use this crate |
|----------------|----------------|
| Persist journal entries with P2P sync | `clasp-journal-defra` |
| Persist router state with sub-100us hot path | `clasp-state-defra` |
| Subscribe to DefraDB changes as CLASP signals | `clasp-defra-bridge` |
| Sync configs between team members | `clasp-config-defra` |
| Store device/user identities in DefraDB | `clasp-registry-defra` |
| Run DefraDB sync through browsers/BLE/Serial | `clasp-defra-transport` |
| Use one keypair for CLASP + DID + libp2p | `clasp-identity` |

## clasp-journal-defra

**Purpose**: Implements the `Journal` trait against DefraDB's HTTP/GraphQL API.

**Key type**: `DefraJournal` -- drop-in replacement for `SqliteJournal` or `MemoryJournal`.

**Also exports**: `DefraClient` -- the shared HTTP client used by all other DefraDB crates.

```rust
use clasp_journal_defra::DefraJournal;
let journal = DefraJournal::connect("http://localhost:9181").await?;
```

**Install**: `cargo add clasp-journal-defra`

## clasp-state-defra

**Purpose**: Write-through cache backed by DefraDB. Hot path stays sub-100us.

**Key type**: `DefraStateStore` with DashMap cache + async background writer.

```rust
use clasp_state_defra::{DefraStateStore, DefraStateConfig};
let store = DefraStateStore::new("http://localhost:9181", DefraStateConfig::default()).await?;
store.set("/path", Value::Float(1.0), "session", None, false, false, None)?;
```

**Install**: `cargo add clasp-state-defra`

## clasp-defra-bridge

**Purpose**: Bidirectional bridge between DefraDB documents and CLASP signals.

**Key types**: `DefraBridge`, `DefraWatcher`, `DefraWriter`, `SignalSender` / `SignalReceiver` traits.

**Address convention**: `/defra/{collection}/{docID}/{field}`

```rust
use clasp_defra_bridge::DefraBridge;
let bridge = DefraBridge::new("http://localhost:9181", vec!["User".into()]);
```

**Install**: `cargo add clasp-defra-bridge`

## clasp-config-defra

**Purpose**: Store router/connection/bridge/rule configs in DefraDB with P2P sync and version history.

**Key type**: `DefraConfigStore` with CRUD for 5 config types + snapshot management.

**ACP module**: `policy.rs` provides the CLASP ACP policy definition (DPI YAML format), `@policy`-annotated schema variants, and policy ID resolution. See [Access Control (ACP)](acp.md) for details.

```rust
use clasp_config_defra::{DefraConfigStore, RouterConfig};
let store = DefraConfigStore::new("http://localhost:9181").await?;
store.save_router(&config).await?;
```

**Install**: `cargo add clasp-config-defra`

## clasp-registry-defra

**Purpose**: Store CLASP entity identities in DefraDB. Implements `EntityStore` trait.

**Key type**: `DefraEntityStore`

```rust
use clasp_registry_defra::DefraEntityStore;
let store = DefraEntityStore::connect("http://localhost:9181").await?;
```

**Install**: `cargo add clasp-registry-defra`

## clasp-defra-transport

**Purpose**: Tunnel DefraDB P2P sync messages over CLASP transports (WebSocket, WebRTC, QUIC, BLE).

**Key types**: `DefraTunnel`, `TunnelMessage` (7 variants: SyncRequest, DagBlock, HeadUpdate, BlockAck, PeerInfo, QueryForward, QueryResponse).

```rust
use clasp_defra_transport::DefraTunnel;
let tunnel = DefraTunnel::new("http://localhost:9181", "my-peer-id");
```

**Install**: `cargo add clasp-defra-transport`

## clasp-identity

**Purpose**: One Ed25519 keypair produces CLASP EntityId + W3C DID + libp2p PeerID.

**Key type**: `Identity`

**secp256k1 feature**: Enable `secp256k1` for DefraDB ACP identity support. Provides `DefraIdentity` which can be derived deterministically from an Ed25519 `Identity` via HKDF, or generated independently.

```rust
use clasp_identity::Identity;
let id = Identity::generate();
println!("{} / {} / {}", id.entity_id(), id.did(), id.peer_id());

// With secp256k1 feature:
#[cfg(feature = "secp256k1")]
{
    use clasp_identity::DefraIdentity;
    let defra_id = DefraIdentity::derive_from(&id).unwrap();
    println!("DefraDB identity: {}", defra_id.to_hex());
}
```

**Install**: `cargo add clasp-identity` (add `--features secp256k1` for DefraDB ACP)
