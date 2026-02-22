---
title: Entity Registry
description: Persistent Ed25519 identity for devices, users, and services
order: 4
---

# Entity Registry

The entity registry gives devices, users, and services persistent Ed25519 identities managed through a REST API. Entities are stored in SQLite and produce `ent_` tokens that the relay validates against the registry on each connection.

## Enable

Entity registry support requires the `registry` feature at build time:

```bash
cargo build --release --features registry
```

At runtime, specify the registry database and enable the auth port:

```bash
clasp-relay --auth-port 7350 --registry-db ./registry.db
```

The registry REST API is served on the same auth port alongside the CPSK auth endpoints.

## Entity Types

| Type | Description |
|------|-------------|
| `Device` | Hardware devices such as sensors, controllers, lights |
| `User` | Human users with persistent identity across sessions |
| `Service` | Software services such as bridges, aggregators, dashboards |
| `Router` | CLASP relay/router instances in federated deployments |

## Entity Lifecycle

Entities transition through three statuses:

```
Active --> Inactive --> Revoked
```

Only entities with `Active` status can authenticate. Setting an entity to `Inactive` temporarily disables access. Setting it to `Revoked` permanently disables access (the entity must be deleted and re-created to restore it).

## Create an Entity

First, generate an Ed25519 keypair for the entity:

```bash
clasp token entity keygen
# Output: public_key: <64 hex chars>, secret saved to entity.key
```

Then create the entity via the REST API. This requires an admin CPSK token:

```bash
curl -X POST http://localhost:7350/api/entities \
  -H 'Authorization: Bearer cpsk_admin...' \
  -H 'Content-Type: application/json' \
  -d '{
    "entity_type": "Device",
    "name": "Temperature Sensor",
    "public_key": "<64 hex chars>",
    "tags": ["sensor", "temperature"],
    "namespaces": ["/sensors/**"],
    "scopes": ["write:/sensors/**", "read:/**"]
  }'
```

Response (`201 Created`):

```json
{
  "id": "clasp:<base58-pubkey-prefix>",
  "entity_type": "Device",
  "name": "Temperature Sensor",
  "public_key": "<64 hex chars>",
  "status": "Active",
  "tags": ["sensor", "temperature"],
  "namespaces": ["/sensors/**"],
  "scopes": ["write:/sensors/**", "read:/**"],
  "created_at": "2026-02-21T00:00:00Z"
}
```

The entity ID uses the format `clasp:<base58-ed25519-pubkey-prefix>`, derived from the entity's public key.

## List and Manage Entities

All management endpoints require an admin CPSK token.

**List entities:**

```bash
curl http://localhost:7350/api/entities?offset=0&limit=100 \
  -H 'Authorization: Bearer cpsk_admin...'
```

**Get by ID:**

```bash
curl http://localhost:7350/api/entities/clasp:abc123 \
  -H 'Authorization: Bearer cpsk_admin...'
```

**Update status:**

```bash
curl -X PUT http://localhost:7350/api/entities/clasp:abc123/status \
  -H 'Authorization: Bearer cpsk_admin...' \
  -H 'Content-Type: application/json' \
  -d '{"status": "Revoked"}'
```

**Delete:**

```bash
curl -X DELETE http://localhost:7350/api/entities/clasp:abc123 \
  -H 'Authorization: Bearer cpsk_admin...'
```

## Mint Entity Tokens

Use the CLI to mint a token signed by the entity's private key:

```bash
clasp token entity mint --key entity.key --id clasp:abc123
# Output: ent_<base64url...>
```

The token contains the entity ID, a timestamp, and an Ed25519 signature. It is encoded as base64url-encoded MessagePack.

You can also inspect a token:

```bash
clasp token entity inspect ent_<token>
# Shows: entity ID, timestamp, signature
```

## Connecting with Entity Tokens

Pass the entity token when building a CLASP client connection.

**JavaScript:**

```javascript
const client = await new ClaspBuilder('ws://localhost:7330')
  .withToken('ent_<token>')
  .connect();
```

**Python:**

```python
client = Clasp('ws://localhost:7330', token='ent_<token>')
await client.connect()
```

**Rust:**

```rust
let client = Clasp::builder("ws://localhost:7330")
    .token("ent_<token>")
    .connect().await?;
```

## Token Validation

When a client presents an `ent_` token, the relay performs these checks:

1. Decode the token and extract the entity ID.
2. Look up the entity in the registry database.
3. Verify the entity's status is `Active`.
4. Verify the Ed25519 signature against the entity's registered public key.
5. Check the token timestamp is within acceptable age.
6. Enforce the entity's configured scopes and namespaces for all subsequent operations.

If any check fails, the connection is rejected with an appropriate error.

## Next Steps

- [CPSK Tokens](cpsk.md) -- simpler register/login/guest authentication
- [Capability Tokens](capability-tokens.md) -- delegatable Ed25519 tokens for offline issuance
- [Security Model](../concepts/security-model.md) -- deep dive into CLASP's security architecture
