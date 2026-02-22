---
title: "Security Model"
description: "CLASP provides security through encryption, authentication, and fine-grained access control using Ed25519-based tokens."
section: explanation
order: 10
---
# Security Model

CLASP provides security through encryption, authentication, and fine-grained access control using Ed25519-based tokens.

## Security Modes

CLASP supports three security modes:

| Mode | Use Case | Features |
|------|----------|----------|
| **Open** | Development, trusted LAN | No encryption, no auth |
| **Encrypted** | Production | TLS encryption |
| **Authenticated** | Multi-user | Encryption + token-based auth with scopes |

## Transport Encryption

### WebSocket (WSS)

Use `wss://` for encrypted connections:

```javascript
// Encrypted
const client = new Clasp('wss://router.example.com:7330');

// Unencrypted (development only!)
const client = new Clasp('ws://localhost:7330');
```

WebSocket uses TLS 1.3 for encryption.

### QUIC

QUIC has mandatory TLS 1.3:

```rust
// Always encrypted
let client = ClaspBuilder::new("quic://router.example.com:7331")
    .connect()
    .await?;
```

### UDP

Raw UDP is unencrypted. For secure UDP:
- Use DTLS wrapper
- Or use QUIC instead
- Or encrypt at application level

## Token Types

CLASP supports three token types, each with a distinct prefix and validation path. All three coexist in the `ValidatorChain`, which dispatches by prefix:

| Token Type | Prefix | Signing | Use Case |
|-----------|--------|---------|----------|
| **CPSK** | `cpsk_` | Pre-shared key (HMAC) | Register/login flow, simple deployments |
| **Capability** | `cap_` | Ed25519 | Delegatable access, offline issuance |
| **Entity** | `ent_` | Ed25519 | Device/service identity, registry-backed |

### CPSK Tokens

Pre-shared key tokens for the register/login flow:

```bash
# Register a user and get a CPSK token
POST /auth/register { "username": "alice", "password": "..." }
# Returns: cpsk_abc123...

# Use in HELLO to authenticate
```

CPSK tokens carry scopes assigned at registration time. The router stores them in-memory or persists them.

### Capability Tokens

Ed25519-signed delegatable tokens (UCAN-style). Created offline with the CLI:

```bash
# Create a root token
clasp token cap create --key root.key --scopes "admin:/**" --expires 30d

# Delegate with narrower scopes
clasp token cap delegate <token> --key child.key --scopes "write:/lights/**"
```

Key properties:
- **Delegation chains** -- tokens can be delegated with attenuated scopes
- **Scope attenuation** -- child scopes must be a subset of parent scopes
- **Expiration clamping** -- child tokens cannot outlive their parent
- **Trust anchors** -- the router validates that the root issuer is trusted
- **Chain depth limits** -- configurable maximum delegation depth

See [Capability Delegation](capability-delegation.md) for detailed delegation rules.

### Entity Tokens

Ed25519-signed identity tokens backed by the entity registry:

```bash
# Generate entity keypair
clasp token entity keygen --out sensor.key --name "Sensor A" --type device

# Mint a token
clasp token entity mint --key sensor.key
```

Key properties:
- **Registry-backed** -- entity must exist and be Active in the store
- **Status lifecycle** -- Active, Suspended, Revoked
- **Namespace scopes** -- entities scoped to namespace patterns
- **Token age** -- optional max age check for freshness

### ValidatorChain

The router composes all three validators into a chain:

```
Token arrives -> ValidatorChain
  -> CpskValidator: cpsk_ prefix? -> validate
  -> CapabilityValidator: cap_ prefix? -> validate
  -> EntityValidator: ent_ prefix? -> validate
  -> No match: reject
```

Each validator returns `NotMyToken` for unrecognized prefixes, allowing the chain to try the next validator. See [Token Validation Flow](token-validation-flow.md) for the full dispatch diagram.

## Scope Format

All token types produce scopes in `action:pattern` format:

```
admin:/**           -- full access to everything
write:/lights/**    -- set/publish under /lights/
read:/sensors/*     -- subscribe to direct children of /sensors/
```

### Actions

| Action | Permits |
|--------|---------|
| `admin` | All operations (read, write, subscribe, admin) |
| `write` | SET and PUBLISH operations |
| `read` | SUBSCRIBE and GET operations |

### Patterns

CLASP address patterns with wildcards:

| Pattern | Matches |
|---------|---------|
| `/lights/room1` | Exact match |
| `/lights/*` | Single segment wildcard |
| `/lights/**` | Multi-segment wildcard (any depth) |

## Session Management

Each connection has a session with associated scopes:

```
Session {
    session_id: "abc123",
    client_name: "My App",
    scopes: ["write:/lights/**", "read:/sensors/**"],
    connected_at: 1704067200,
    subscriptions: ["/lights/**", "/sensors/**"],
    federation_peer: false,
}
```

Sessions enable:
- Identifying who wrote what
- Revoking access (close session)
- Rate limiting per client
- Scope enforcement on every operation

## Rate Limiting

Routers limit request rates per client:

```rust
let config = RouterConfig {
    rate_limiting_enabled: true,
    max_messages_per_second: 500,
    ..Default::default()
};
```

When a client exceeds the rate limit, excess messages are dropped and a warning is logged. Buffer overflow notifications (ERROR 503) are sent after 100 drops within 10 seconds, rate-limited to 1 per 10 seconds per session.

## Federation Security

Federation peers have additional security constraints:

| Mechanism | Purpose |
|-----------|---------|
| **Namespace restriction** | Peers can only access data within their declared namespaces |
| **Scope enforcement** | In authenticated mode, peers need scopes covering their namespaces |
| **Resource limits** | Max 1,000 patterns per peer, max 10,000 revision entries |
| **Origin tracking** | Loop prevention via origin field on forwarded messages |
| **Feature detection** | Non-federation sessions get 403 for FederationSync |

See [Federation Message Sequence](federation-message-sequence.md) for enforcement details.

## Threat Model

### Eavesdropping

**Threat:** Attackers read messages.
**Mitigation:** Use TLS/DTLS encryption (WSS, QUIC).

### Man-in-the-Middle

**Threat:** Attacker intercepts and modifies messages.
**Mitigation:** TLS certificate verification.

### Unauthorized Access

**Threat:** Unauthorized clients connect.
**Mitigation:** Token-based auth (CPSK, Capability, or Entity tokens) with scope enforcement.

### Token Forgery

**Threat:** Attacker creates fake capability or entity tokens.
**Mitigation:** Ed25519 signature verification. Capability tokens require a trusted root issuer (trust anchor). Entity tokens require the entity to exist in the registry.

### Scope Escalation

**Threat:** Client attempts operations outside their scopes.
**Mitigation:** Per-operation scope checks. Capability delegation enforces attenuation (child scopes must be subset of parent).

### Replay Attacks

**Threat:** Attacker replays captured tokens.
**Mitigation:** Token expiration, capability token nonces (UUID v4), entity token age checks.

### Federation Namespace Hijacking

**Threat:** Federation peer claims namespaces it doesn't own.
**Mitigation:** Scope checks in authenticated mode. Namespace restriction on RequestSync and RevisionVector.

### Resource Exhaustion

**Threat:** Federation peer floods with patterns or revision entries.
**Mitigation:** `MAX_FEDERATION_PATTERNS = 1,000` and `MAX_REVISION_ENTRIES = 10,000` limits.

### Parameter Manipulation

**Threat:** Client sets values outside allowed range.
**Mitigation:** Scope-based write restrictions.

### DoS (Rate Flooding)

**Threat:** Client sends excessive messages.
**Mitigation:** Per-client rate limiting.

## Security Recommendations

### Development

- `ws://` is fine for localhost
- No tokens needed (Open mode)
- Disable in production

### Production (Trusted Network)

- Use `wss://` always
- Verify TLS certificates
- Rate limit clients
- Use CPSK tokens for simple auth

### Production (Untrusted Network)

- Use `wss://` with valid certificates
- Use capability tokens for delegatable access
- Use entity tokens for device identity
- Short token expiration (< 24h)
- Configure trust anchors for capability validation
- Log security events

### Multi-Tenant / Multi-Site

- Namespace isolation per tenant
- Federation with namespace restriction
- Separate trust anchors per site
- Entity registry with per-namespace scopes
- Audit logging

## Key Management

### CLI Key Generation

```bash
# Generate Ed25519 keypair (hex-encoded, 0600 permissions)
clasp key generate --out root.key

# Show public key
clasp key show root.key

# Show in did:key format
clasp key show root.key --format did
```

### Trust Anchor Configuration

Trust anchors are Ed25519 public keys that the router trusts as root issuers for capability tokens:

```bash
clasp-relay --trust-anchor ./root.key --cap-max-depth 5
```

### Admin Bootstrap

The relay supports automatic admin token bootstrap:

```bash
# If file exists: reads token. If not: generates and writes with 0600 permissions.
clasp-relay --admin-token ./admin.token
```

The admin token is registered with `admin:/**` scope, solving the chicken-and-egg problem of needing an admin token to use the registry API.

## See Also

- [Token Validation Flow](token-validation-flow.md) -- ValidatorChain dispatch details
- [Capability Delegation](capability-delegation.md) -- Delegation chains and scope attenuation
- [Federation Message Sequence](federation-message-sequence.md) -- Federation security enforcement
- [Distributed Architecture](distributed-architecture.md) -- System architecture overview
