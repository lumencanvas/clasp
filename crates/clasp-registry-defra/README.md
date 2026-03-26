# clasp-registry-defra

DefraDB backend for the CLASP entity registry. Implements the `EntityStore` trait from `clasp-registry`, enabling P2P-synced device, user, and service identities.

## Usage

```rust
use clasp_registry_defra::DefraEntityStore;
use clasp_registry::EntityStore;

let store = DefraEntityStore::connect("http://localhost:9181").await?;

// CRUD
store.create(&entity).await?;
let found = store.get(&entity_id).await?;
store.update_status(&entity_id, EntityStatus::Revoked).await?;

// Search
let devices = store.find_by_tag("lighting").await?;
let local = store.find_by_namespace("/venue/main").await?;
```

## What syncs

When two CLASP routers use DefraDB-backed entity stores pointed at peered DefraDB nodes, entity registrations, status changes, and revocations propagate automatically via Merkle CRDTs.

## License

MIT OR Apache-2.0
