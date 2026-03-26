---
title: Configuration Sync
description: P2P configuration management with version history
order: 4
---

# Configuration Sync

The `clasp-config-defra` crate stores CLASP configurations in DefraDB, enabling P2P sync between team members, version history with rollback, and snapshot management.

## Config Types

| Type | What it stores |
|------|---------------|
| `RouterConfig` | Host, port, transports, security, TTL, features |
| `ConnectionConfig` | Router URL, transport, token, reconnect settings |
| `BridgeConfig` | Protocol, source/target addresses, mappings |
| `RuleConfig` | Triggers, conditions, actions, cooldown |
| `ConfigSnapshot` | Full capture of all config types above |

## Usage

```rust
use clasp_config_defra::{DefraConfigStore, RouterConfig, ConfigSnapshot};

let store = DefraConfigStore::new("http://localhost:9181").await?;

// Save a router config (upserts by config_id)
let config = RouterConfig::new("main-router", "Studio Router", "alice");
store.save_router(&config).await?;

// List configs by owner
let my_routers = store.list_routers_by_owner("alice").await?;

// Take a snapshot of the entire config
let snapshot = ConfigSnapshot { ... };
store.save_snapshot(&snapshot).await?;

// Roll back: load a previous snapshot
let old = store.get_snapshot("snapshot-id").await?;

// JSON import/export (compatible with file-based config)
let json = store.export_json().await?;
store.import_json(&json, "alice").await?;
```

## P2P Config Sync

When two team members use `DefraConfigStore` pointed at peered DefraDB nodes, config changes propagate automatically:

1. Alice saves a router config on her machine
2. DefraDB replicates the document to Bob's node via Merkle CRDTs
3. Bob's `DefraConfigStore` sees the new config on its next query

Concurrent edits merge deterministically: DefraDB's LWW-Register picks the latest write by timestamp.

## Version History

DefraDB stores every document mutation as a node in a Merkle DAG. Use `config_history()` to walk previous versions:

```rust
let history = store.config_history("main-router", "ClaspRouterConfig").await?;
// Returns a Vec of commit objects with previous state
```

This enables rollback to any prior configuration state.

## Bridge App Integration

The CLASP Bridge desktop app exposes DefraDB config sync through its settings panel:

1. Enable DefraDB in the DEFRA tab
2. Enter the DefraDB URL (e.g., `http://localhost:9181`)
3. Health indicator shows connection status
4. Export/Import buttons sync configs with DefraDB

The Electron IPC handlers (`defra-config-export`, `defra-config-import`, `defra-health-check`) are available to the renderer via `window.clasp.defraConfigExport()`, etc.
