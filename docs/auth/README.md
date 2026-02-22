---
title: Auth & Security
description: Authentication, authorization, and encryption in CLASP
order: 1
---

# Auth & Security

CLASP supports three token types for authentication and TLS for transport encryption. Auth is opt-in -- disable it for trusted LANs, enable it for production deployments.

## Token Types at a Glance

| Type | Prefix | Signing | Use Case | Enable Flag |
|------|--------|---------|----------|-------------|
| CPSK | `cpsk_` | Pre-shared key | Register/login/guest flow | `--auth-port 7350` |
| Capability | `cap_` | Ed25519 | Delegatable access, offline issuance | `--trust-anchor anchor.pub` |
| Entity | `ent_` | Ed25519 | Device/service persistent identity | `--registry-db registry.db` |

## Scope Format

Every token carries scopes in the format `action:pattern`.

### Actions

| Action | Permits | Implies |
|--------|---------|---------|
| `read` | subscribe, get | -- |
| `write` | set, publish | read |
| `admin` | full access including registry API | write, read |

The hierarchy is `admin > write > read`. A token with `write` scope implicitly has `read` access to the same paths.

### Patterns

Patterns use path-style matching with two wildcard forms:

| Pattern | Matches |
|---------|---------|
| `read:/**` | Read access to all paths |
| `write:/lights/**` | Write access to `/lights/` and all descendants (recursive) |
| `read:/sensors/*` | Read access to one level under `/sensors/` |
| `write:/app/alice/profile` | Write access to one exact path |

## ValidatorChain

The relay dispatches incoming tokens by prefix to the appropriate validator:

- `cpsk_` --> CPSK validator
- `cap_` --> Capability validator
- `ent_` --> Entity validator
- Unknown prefix --> rejected

All three validators can run simultaneously. The relay inspects the token prefix and routes to the correct one without configuration.

## Quick Example

Start a relay with CPSK auth, register a user, and connect:

```bash
# Start relay with auth
clasp-relay --auth-port 7350

# Register a user
curl -X POST http://localhost:7350/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"username": "alice", "password": "secret", "scopes": ["read:/**", "write:/app/**"]}'
# Returns: {"token": "cpsk_abc123...", "session_id": "...", "scopes": [...]}
```

```javascript
// Connect with token
const client = await new ClaspBuilder('ws://localhost:7330')
  .withToken('cpsk_abc123...')
  .connect();
```

## Security Modes

| Mode | TLS | Auth | When to Use |
|------|-----|------|-------------|
| Open | No | No | Local development, trusted LAN |
| Encrypted | Yes | No | Trusted users, untrusted network |
| Authenticated | Yes | Yes | Production, public-facing deployments |

In open mode, any client can connect and access all paths. Adding TLS encrypts traffic but does not restrict access. Adding tokens on top of TLS provides both encryption and access control.

## Next Steps

- [CPSK Tokens](cpsk.md) -- register/login/guest authentication
- [Capability Tokens](capability-tokens.md) -- Ed25519 delegatable tokens
- [Entity Registry](entity-registry.md) -- persistent device and service identity
- [TLS Setup](tls.md) -- transport encryption
