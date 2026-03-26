# clasp-config-defra

DefraDB-backed configuration storage for CLASP. Stores router, connection, bridge, and rule configs with P2P sync, version history, and snapshot management.

## Usage

```rust
use clasp_config_defra::{DefraConfigStore, RouterConfig};

let store = DefraConfigStore::new("http://localhost:9181").await?;

// Save a router config (upsert by config_id)
let config = RouterConfig::new("main-router", "Studio Router", "alice");
store.save_router(&config).await?;

// Snapshot the entire config for versioning
let snapshot = store.save_snapshot(&full_snapshot).await?;

// Import/export JSON (compatible with file-based config)
let json = store.export_json().await?;
store.import_json(&json, "alice").await?;
```

## Config types

- `RouterConfig`: Host, port, transports, security, TTL, features
- `ConnectionConfig`: Router URL, transport, token, reconnect
- `BridgeConfig`: Protocol, source/target addresses, mappings
- `RuleConfig`: Triggers, conditions, actions, cooldown
- `ConfigSnapshot`: Full config capture with all of the above

## P2P sync

When two team members use DefraDB-backed config stores pointed at peered DefraDB nodes, config changes propagate automatically. Each config document is a CRDT -- concurrent edits merge deterministically.

## Version history

DefraDB stores every document mutation as a node in a Merkle DAG. Use `config_history()` to walk previous versions of any config, enabling rollback to any prior state.

## License

MIT OR Apache-2.0
