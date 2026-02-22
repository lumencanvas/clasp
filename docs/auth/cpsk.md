---
title: CPSK Tokens
description: Register, login, and guest authentication with CPSK tokens
order: 2
---

# CPSK Tokens

CPSK (Client Pre-Shared Key) tokens are the simplest auth method in CLASP. Enable an HTTP auth server on the relay, register users with passwords, and connect with the returned tokens. Passwords are hashed with argon2 before storage.

## Enable

Start the relay with the `--auth-port` flag to enable the auth HTTP server:

```bash
clasp-relay --auth-port 7350
```

Additional options:

| Flag | Default | Description |
|------|---------|-------------|
| `--auth-db` | `relay-auth.db` | SQLite database file for users and tokens |
| `--cors-origin` | none | Comma-separated allowed origins for CORS |
| `--token-ttl` | `86400` | Token lifetime in seconds (default 1 day) |
| `--admin-token` | none | Path to admin token file (auto-generated if missing) |

## Register

Create a new user account with scoped access.

```bash
curl -X POST http://localhost:7350/auth/register \
  -H 'Content-Type: application/json' \
  -d '{
    "username": "alice",
    "password": "secure-password",
    "scopes": ["read:/**", "write:/app/alice/**"]
  }'
```

Response:

```json
{
  "token": "cpsk_<uuid>",
  "session_id": "...",
  "scopes": ["read:/**", "write:/app/alice/**"]
}
```

The password is hashed with argon2 before storage. The plaintext password is never persisted.

## Login

Authenticate an existing user.

```bash
curl -X POST http://localhost:7350/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username": "alice", "password": "secure-password"}'
```

Returns the same format as register. A new token is issued on each login.

## Guest

Create a guest session with no username or password.

```bash
curl -X POST http://localhost:7350/auth/guest \
  -H 'Content-Type: application/json' \
  -d '{"scopes": ["read:/**"]}'
```

Guest tokens are useful for read-only public access where you want to limit what paths a client can see without requiring credentials.

## Connecting with a Token

Pass the token when building a CLASP client connection.

**JavaScript:**

```javascript
const client = await new ClaspBuilder('ws://localhost:7330')
  .withToken('cpsk_abc123...')
  .connect();
```

**Python:**

```python
client = Clasp('ws://localhost:7330', token='cpsk_abc123...')
await client.connect()
```

**Rust:**

```rust
let client = Clasp::builder("ws://localhost:7330")
    .token("cpsk_abc123...")
    .connect().await?;
```

## Scope Enforcement

Every operation is checked against the token's scopes at the relay. If a client attempts an operation outside its allowed scopes, the relay rejects it with an error.

```javascript
// Token has scopes: ["read:/**", "write:/app/alice/**"]

await client.set('/app/alice/status', 'online');   // allowed
await client.get('/sensors/temperature');            // allowed (read://**)
await client.set('/admin/config', 'value');          // rejected -- no write scope for /admin/**
```

The relay enforces the action hierarchy: `admin > write > read`. A `write` scope implicitly grants `read` access to the same paths. An `admin` scope grants both `write` and `read`.

## Rate Limits

The auth HTTP endpoints are rate-limited to prevent brute-force attacks:

| Endpoint | Default Limit |
|----------|---------------|
| `/auth/login` | 5 attempts per 60 seconds |
| `/auth/register` | 10 attempts per 60 seconds |

Rate limits are configurable via the app config file. See [App Config](../reference/router-config.md) for details.

## Admin Token

The `--admin-token` flag points to a file containing (or that will contain) an admin token with `admin:/**` scope:

```bash
clasp-relay --auth-port 7350 --admin-token ./admin.token
```

If the file does not exist, the relay generates a new admin token and writes it to the file with `0600` permissions. This token is required for managing the entity registry and other admin operations.

## Next Steps

- [Capability Tokens](capability-tokens.md) -- Ed25519 delegatable tokens for offline issuance
- [Entity Registry](entity-registry.md) -- persistent identity for devices and services
- [Router Config](../reference/router-config.md) -- scope templates and rate limit configuration
