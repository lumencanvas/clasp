# clasp-registry

Persistent Ed25519 identity registry for devices, users, services, and routers in CLASP.

## Features

- **Entity Identity** - Ed25519 keypairs with globally unique `clasp:` IDs
- **Entity Types** - Device, User, Service, Router
- **Status Lifecycle** - Active, Suspended, Revoked
- **Token Generation** - Ed25519-signed entity tokens (`ent_` prefix)
- **Pluggable Storage** - In-memory or SQLite backends
- **ValidatorChain Integration** - Works alongside CPSK and Capability validators
- **Namespace-Based Scopes** - Entities can be scoped to namespace patterns

## Installation

```toml
[dependencies]
clasp-registry = "3.5"

# With SQLite persistence
clasp-registry = { version = "3.5", features = ["sqlite"] }
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `sqlite` | Enables `SqliteEntityStore` for persistent storage |

## Usage

### Generate an Entity Keypair

```rust
use clasp_registry::{EntityKeypair, EntityType};

let keypair = EntityKeypair::generate()?;
println!("Entity ID: {}", keypair.entity_id); // clasp:4j8Ygfakd8xZ9q7m...

// Create an Entity from the keypair
let entity = keypair.to_entity(EntityType::Device, "Sensor A".to_string());
```

### Store and Retrieve Entities

```rust
use clasp_registry::{MemoryEntityStore, EntityStore, EntityId};

let store = MemoryEntityStore::new();

// Create
store.create(&entity).await?;

// Retrieve
let found = store.get(&entity.id).await?;
assert!(found.is_some());

// Find by public key
let by_key = store.find_by_public_key(keypair.public_key_bytes()).await?;

// Find by tag or namespace
let sensors = store.find_by_tag("sensor").await?;
let site_a = store.find_by_namespace("/site-a").await?;

// List with pagination
let page = store.list(0, 20).await?;

// Update status (suspend, revoke)
use clasp_registry::EntityStatus;
store.update_status(&entity.id, EntityStatus::Suspended).await?;
```

### SQLite Persistence

```rust
#[cfg(feature = "sqlite")]
use clasp_registry::SqliteEntityStore;

let store = SqliteEntityStore::open("entities.db")?;
// Same EntityStore API as MemoryEntityStore
```

### Generate and Validate Tokens

```rust
use clasp_registry::{generate_token, EntityValidator};
use std::sync::Arc;

// Generate a token from a keypair
let token = generate_token(&keypair)?; // "ent_<base64url(msgpack)>"

// Set up validation
let store = Arc::new(MemoryEntityStore::new());
store.create(&entity).await?;

let validator = EntityValidator::new(store)
    .with_max_token_age(3600); // reject tokens older than 1 hour

// Implements clasp_core::security::TokenValidator
use clasp_core::security::TokenValidator;
match validator.validate(&token) {
    clasp_core::security::ValidationResult::Valid(info) => {
        println!("Entity: {}", info.subject.unwrap());
        println!("Scopes: {:?}", info.scopes);
    }
    other => println!("Validation failed: {:?}", other),
}
```

## Token Wire Format

Entity tokens use the `ent_` prefix followed by URL-safe base64-encoded MessagePack:

```
ent_<base64url(msgpack(EntityTokenPayload))>
```

The payload contains the entity ID, a timestamp, and an Ed25519 signature over both.

## Configuration Reference

### Entity

| Field | Type | Description |
|-------|------|-------------|
| `id` | `EntityId` | Globally unique ID (`clasp:<base58>`) |
| `entity_type` | `EntityType` | `Device`, `User`, `Service`, or `Router` |
| `name` | `String` | Human-readable name |
| `public_key` | `Vec<u8>` | Ed25519 public key (32 bytes, hex in JSON) |
| `created_at` | `SystemTime` | Creation timestamp |
| `metadata` | `HashMap<String, String>` | Arbitrary key-value metadata |
| `tags` | `Vec<String>` | Searchable tags |
| `namespaces` | `Vec<String>` | Namespace patterns (converted to scopes) |
| `scopes` | `Vec<String>` | Explicit `action:pattern` scopes |
| `status` | `EntityStatus` | `Active`, `Suspended`, or `Revoked` |

### EntityId Format

```
clasp:<base58(first 16 bytes of Ed25519 public key)>
```

### Scope Resolution

When validating, scopes are resolved in order:

1. If `entity.scopes` is non-empty, use them as-is
2. Otherwise, convert `entity.namespaces` to scopes: `/lights` becomes `admin:/lights/**`

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

---

Maintained by [LumenCanvas](https://lumencanvas.studio)
