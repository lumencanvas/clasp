---
title: Security Model
description: Threat model, token types, and scope enforcement in CLASP
order: 3
---

# Security Model

CLASP's security model covers four layers: encryption (TLS/DTLS on the transport), authentication (three token types that identify who is connecting), authorization (scoped permissions that control what each session can do), and data isolation (app config rules that control what each session can see).

These layers are independent and additive. You can run with none of them during development, enable TLS for a trusted LAN, or stack all four for a multi-tenant production deployment.

## Security Modes

CLASP supports three security modes, chosen at deployment time:

| Mode | Encryption | Auth | Use Case |
|---|---|---|---|
| Open | None | None | Local development, trusted isolated network |
| Encrypted | TLS | None | Trusted network where data must be protected in transit |
| Authenticated | TLS | Tokens | Production, multi-user, untrusted network |

Open mode is the default. The relay starts with no TLS and no auth unless you pass the relevant flags. This is intentional -- security should be easy to add, but it should not block getting started.

In authenticated mode, every connecting client must present a valid token. The router rejects connections without tokens and enforces scoped permissions on every operation for the lifetime of the session.

## Token Types

CLASP supports three token types, each designed for a different scenario. They can all be active simultaneously -- the router dispatches to the correct validator based on the token prefix.

### CPSK Tokens (`cpsk_`)

Pre-shared key authentication. The simplest token type, designed for user-facing applications.

**Flow:**
1. Client sends username and password to the auth HTTP endpoint (`POST /auth/register` or `POST /auth/login`).
2. Server hashes the password with argon2, stores the credential, and returns a `cpsk_` token with scoped permissions.
3. Client connects to the router WebSocket with the token in the Hello message.
4. Router validates the token against the auth database and creates a session with the declared scopes.

Guest tokens (`POST /auth/guest`) skip the password step and issue tokens with restricted scopes, useful for read-only dashboards or public displays.

**Best for:** Web apps, mobile apps, any scenario where users log in with credentials.

### Capability Tokens (`cap_`)

Ed25519-signed tokens following the UCAN (User Controlled Authorization Networks) model. These tokens are self-contained -- the router verifies them cryptographically without a database lookup.

**Flow:**
1. A trust anchor key pair is configured on the relay (`--trust-anchor`).
2. The trust anchor signs a root capability token with broad scopes.
3. The root token holder can delegate narrower tokens by signing new tokens with their own key, embedding the parent token in the chain.
4. The router verifies the entire chain back to the trust anchor and enforces the narrowest scope in the chain.

**Delegation rules:**
- Scopes can only be narrowed, never broadened. A token with `write:/lighting/**` can delegate `write:/lighting/zone-1/**` but not `write:/audio/**`.
- Expiration is clamped: a child token cannot outlive its parent.
- Action hierarchy is respected: a `read` parent cannot delegate `write`.
- Maximum delegation depth is enforced to prevent unbounded chains.

**Best for:** IoT devices, third-party integrations, offline token issuance, scenarios where the token issuer and the router are not always connected.

### Entity Tokens (`ent_`)

Registry-backed persistent identity tokens for devices and services. Unlike CPSK tokens (tied to user credentials) and capability tokens (tied to delegation chains), entity tokens represent long-lived identities managed through a REST API.

**Flow:**
1. The entity registry is enabled on the relay (`--registry-db`).
2. An admin registers a device via the registry REST API, receiving an `ent_` token.
3. The device connects with its entity token.
4. The router validates the token against the registry and checks the entity's status (Active, Inactive, or Revoked).

**Lifecycle:** Entities can be deactivated (temporarily denied access) or revoked (permanently denied access) through the registry API without rotating tokens or restarting the router.

**Best for:** Permanent installations, device fleets, services that need stable identity across restarts and network changes.

## ValidatorChain

The router uses a ValidatorChain to dispatch tokens to the correct validator. All three validators can be active simultaneously. Dispatch is based on the token prefix:

```
Token arrives with Hello message
  │
  ├─ prefix "cpsk_" ──> CpskValidator
  │                       └─ Lookup in auth database
  │                       └─ Return scopes from stored credential
  │
  ├─ prefix "cap_"  ──> CapabilityValidator
  │                       └─ Decode token chain
  │                       └─ Verify each Ed25519 signature
  │                       └─ Check expiration at each level
  │                       └─ Compute intersection of all scopes
  │
  ├─ prefix "ent_"  ──> EntityValidator
  │                       └─ Lookup in registry database
  │                       └─ Check entity status (Active?)
  │                       └─ Return scopes from registry entry
  │
  └─ unknown prefix ──> Reject connection
```

If no validators are configured (open mode), the router accepts all connections with unrestricted scopes.

## Scope Format

Every token carries a list of scopes that define what the session is allowed to do. Scopes use the format `action:pattern`.

### Actions

| Action | Allows | Implies |
|---|---|---|
| `read` | `subscribe`, `get`, receive snapshots | -- |
| `write` | `set`, `publish`, `emit`, plus everything `read` allows | `read` |
| `admin` | Everything `write` allows, plus registry API access, router management | `write`, `read` |

The hierarchy means a `write` scope implicitly grants `read` for the same pattern. You do not need to declare both.

### Patterns

Patterns use path-style matching:

| Pattern | Matches |
|---|---|
| `/lighting/zone-1/brightness` | Exactly that address |
| `/lighting/zone-1/*` | Any single segment under `/lighting/zone-1/` |
| `/lighting/**` | Any address starting with `/lighting/`, at any depth |
| `/**` | Everything (used for admin tokens) |

### Examples

| Scope | Effect |
|---|---|
| `read:/lighting/**` | Can subscribe to and get any lighting data. Cannot write. |
| `write:/lighting/zone-1/*` | Can read and write any param directly under zone-1. Cannot access zone-2. |
| `write:/chat/room/*/messages` | Can post messages to any chat room. Cannot modify room settings. |
| `admin:/**` | Full access to everything, including registry API. |
| `read:/**`, `write:/sensors/my-device/**` | Can read everything, but can only write to own sensor namespace. |

### Enforcement

Scopes are checked on every operation, not just at connection time. If a client with `read:/lighting/**` sends a SET message to `/lighting/zone-1/brightness`, the router rejects the operation with an authorization error. The state is not modified, no subscribers are notified, and the client receives an error response.

## Session Lifecycle

```
Client                              Router
  │                                    │
  │  1. Connect (transport handshake)  │
  │ ──────────────────────────────────>│
  │                                    │
  │  2. Hello (token, client_name)     │
  │ ──────────────────────────────────>│
  │                                    │  3. Validate token
  │                                    │     (ValidatorChain)
  │                                    │  4. Create session:
  │                                    │     - session_id
  │                                    │     - client_name
  │                                    │     - scopes
  │                                    │     - connected_at
  │                                    │     - subscriptions: []
  │  5. Welcome (session_id, config)   │
  │ <──────────────────────────────────│
  │                                    │
  │  6. Subscribe / Set / Get / ...    │
  │ ──────────────────────────────────>│  7. Check scopes
  │                                    │     for EVERY operation
  │  ...                               │
  │                                    │
  │  8. Disconnect                     │
  │ ──────────────────────────────────>│  9. Clean up session:
  │                                    │     - Remove subscriptions
  │                                    │     - Release locks
  │                                    │     - Notify presence
```

The session is the unit of authorization. Every subscription and every write operation is checked against the session's scopes. If the token is revoked or expires during the session (relevant for capability and entity tokens), the router terminates the session.

## Capability Delegation

Capability tokens form a chain of trust from a root authority down to the final holder. Each link in the chain narrows the permissions:

```
Trust Anchor (configured on relay)
  │
  │  Signs root token
  │  Scopes: admin:/**
  │  Expires: 2027-01-01
  │
  v
Root Token (held by system admin)
  │
  │  Signs delegated token
  │  Scopes: write:/lighting/**     <-- narrowed from admin:/**
  │  Expires: 2026-06-01            <-- clamped from 2027-01-01
  │
  v
Delegated Token (held by lighting operator)
  │
  │  Signs further delegation
  │  Scopes: write:/lighting/zone-1/**   <-- narrowed again
  │  Expires: 2026-03-01                 <-- clamped again
  │
  v
Device Token (held by lighting fixture)
```

The router verifies this chain by:

1. Checking the final signature against the delegating key.
2. Walking up the chain, verifying each signature.
3. Confirming the root is signed by the configured trust anchor.
4. Computing the intersection of all scopes in the chain (the narrowest wins).
5. Using the earliest expiration in the chain.
6. Checking that the chain depth does not exceed the configured maximum.

This enables offline token issuance. A lighting operator can issue tokens to fixtures in the field without contacting the router. The router verifies the chain cryptographically when the fixture connects.

## Data Isolation with App Config

Token scopes control what operations a session can perform. App config goes further by controlling what data a session can see, even for addresses it has read access to.

App config provides three mechanisms:

### Write Rules

Server-side validation of write operations beyond scope checks. Write rules can enforce constraints like:

- Only the session that created a resource can modify it (ownership).
- Values must fall within a valid range.
- Certain fields are immutable after creation.
- Write rate is limited per address or per session.

Write rules are evaluated after scope checks and before the state store is updated. A rejected write returns an error to the client.

### Snapshot Visibility

Controls which state entries are included when a client receives a snapshot (on subscribe or late-join sync). Visibility rules can:

- Hide entries created by other users (per-user isolation).
- Restrict entries to sessions with specific scopes.
- Filter entries by address pattern beyond what scopes allow.

This creates data isolation without requiring separate routers. A multi-tenant chat application can use visibility rules to ensure that users in room A never receive messages from room B, even if their token scopes technically cover both.

### Snapshot Transforms

Modifies state entries before delivery to specific clients. Transforms can:

- Redact sensitive fields (e.g., remove email addresses from user profiles).
- Summarize data (e.g., replace a full user list with a count).
- Convert data formats for specific client types.

Transforms are applied after visibility filtering and before delivery. The state store retains the full data; only the delivered snapshot is modified.

## Threat Model

| Threat | Mitigation |
|---|---|
| Eavesdropping | TLS encryption on WebSocket and QUIC transports. DTLS on UDP. |
| Man-in-the-middle | TLS certificate verification. Clients can pin certificates. |
| Unauthorized access | Token authentication required in authenticated mode. Connections without valid tokens are rejected. |
| Token forgery | Ed25519 signature verification for capability and entity tokens. HMAC verification for CPSK tokens. |
| Scope escalation | Per-operation scope checks. A valid token with `read` scopes cannot perform `write` operations. |
| Replay attacks | Token expiration enforced. Capability tokens include nonces in delegation chains. |
| Federation hijacking | Namespace ownership enforced. A federated peer can only write to its declared namespace prefix. |
| Resource exhaustion | Configurable session limits (max connections per IP, max subscriptions per session). Rate limiting per client. |
| Data leakage | Snapshot visibility rules and transforms. Sensitive data redacted before delivery. |

## Recommendations by Environment

| Environment | Recommended Setup |
|---|---|
| Local development | Open mode, no TLS. Fastest iteration. |
| Trusted LAN (studio, venue) | TLS enabled, CPSK auth optional. Protects data in transit. |
| Production, single tenant | TLS + CPSK auth + app config write rules + journal. Full auth with server-side validation. |
| Production, multi-tenant | TLS + capability tokens + entity registry + app config (write rules + visibility + transforms) + federation with namespace isolation. Full stack. |

Start with the simplest setup that meets your security requirements. Each layer is additive -- you can enable TLS without auth, add CPSK auth later, and add app config after that, without changing client code.

## Next Steps

- [Architecture](./architecture.md) -- How the crates and components fit together.
- [Why CLASP](./why-clasp.md) -- The problems CLASP solves and comparisons with alternatives.
- [Capability Tokens](../auth/capability-tokens.md) -- Step-by-step guide to issuing and delegating capability tokens.
- [App Config Reference](../reference/app-config-schema.md) -- Configuration reference for write rules and visibility.
