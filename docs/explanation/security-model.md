# Security Model

CLASP provides security through encryption, authentication, and fine-grained access control.

## Security Modes

CLASP supports three security modes:

| Mode | Use Case | Features |
|------|----------|----------|
| **Open** | Development, trusted LAN | No encryption, no auth |
| **Encrypted** | Production | TLS encryption |
| **Authenticated** | Multi-user | Encryption + capability tokens |

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

## Capability Tokens

Capability tokens provide fine-grained access control using JWT:

```javascript
{
  "iss": "clasp:myapp",           // Issuer
  "sub": "user:alice",            // Subject (user identity)
  "iat": 1704067200,              // Issued at
  "exp": 1704153600,              // Expires
  "sf": {                         // CLASP capabilities
    "read": ["/public/**", "/user/alice/**"],
    "write": ["/user/alice/**"],
    "constraints": {
      "/user/alice/volume": {
        "range": [0, 100]
      }
    }
  }
}
```

### Token Fields

| Field | Description |
|-------|-------------|
| `iss` | Token issuer (your app/service) |
| `sub` | User identity |
| `iat` | Issue timestamp |
| `exp` | Expiration timestamp |
| `sf.read` | Address patterns client can read |
| `sf.write` | Address patterns client can write |
| `sf.constraints` | Value constraints per address |

### Using Tokens

**Client side:**
```javascript
const client = await new ClaspBuilder('wss://router:7330')
  .withToken('eyJhbGciOiJIUzI1NiIs...')
  .connect();
```

**Router validates:**
1. Token signature (using shared secret or public key)
2. Expiration
3. Read/write permissions for each operation
4. Value constraints

### Constraints

Tokens can limit values:

```javascript
"constraints": {
  "/lights/*/brightness": {
    "range": [0, 1],        // Allowed value range
    "maxRate": 60           // Max updates per second
  },
  "/system/**": {
    "readonly": true        // Can read but not write
  }
}
```

## Zero-Config Pairing

For local setups without PKI:

1. **Router displays code:**
   ```
   Pairing code: 847291
   ```

2. **Client enters code:**
   ```javascript
   const client = await new ClaspBuilder('ws://192.168.1.42:7330')
     .withPairingCode('847291')
     .connect();
   ```

3. **Session established with encryption**

This uses the pairing code to derive a shared secret for the session.

## Address-Based Access Control

Without tokens, access can be controlled by address patterns:

```yaml
# Router config
security:
  default: deny
  rules:
    - pattern: "/public/**"
      access: read-write
    - pattern: "/admin/**"
      access: deny
    - pattern: "/**"
      access: read-only
```

## Session Management

Each connection has a session:

```javascript
{
  sessionId: "abc123",
  clientName: "My App",
  connectedAt: 1704067200,
  capabilities: { ... },
  subscriptions: ["/lights/**", "/audio/**"]
}
```

Sessions enable:
- Identifying who wrote what
- Revoking access
- Rate limiting per client

## Rate Limiting

Routers can limit request rates:

```yaml
security:
  rateLimit:
    default: 1000/minute
    perAddress:
      "/stream/**": 10000/minute    # Higher for streams
      "/admin/**": 10/minute        # Lower for admin
```

## Threat Model

### Eavesdropping

**Threat:** Attackers read messages.
**Mitigation:** Use TLS/DTLS encryption (WSS, QUIC).

### Man-in-the-Middle

**Threat:** Attacker intercepts and modifies messages.
**Mitigation:** TLS certificate verification.

### Unauthorized Access

**Threat:** Unauthorized clients connect.
**Mitigation:** Capability tokens with read/write restrictions.

### Replay Attacks

**Threat:** Attacker replays captured messages.
**Mitigation:** Timestamps and sequence numbers in protocol.

### Parameter Manipulation

**Threat:** Client sets values outside allowed range.
**Mitigation:** Value constraints in capability tokens.

### DoS (Rate Flooding)

**Threat:** Client sends excessive messages.
**Mitigation:** Per-client rate limiting.

## Security Recommendations

### Development

- `ws://` is fine for localhost
- No tokens needed
- Disable in production!

### Production (Trusted Network)

- Use `wss://` always
- Verify certificates
- Rate limit clients

### Production (Untrusted Network)

- Use `wss://` with valid certificates
- Require capability tokens
- Implement value constraints
- Log security events
- Short token expiration (< 24h)

### Multi-Tenant

- Namespace isolation per tenant
- Separate tokens per tenant
- Audit logging
- Token refresh mechanism

## Implementation Notes

### Token Signing

Tokens can be signed with:
- **HMAC (HS256):** Shared secret between router and token issuer
- **RSA (RS256):** Asymmetric keys, router only needs public key
- **ECDSA (ES256):** Smaller signatures than RSA

### Key Management

For HMAC:
```yaml
security:
  tokenSecret: "${TOKEN_SECRET}"  # From environment
```

For RSA/ECDSA:
```yaml
security:
  tokenPublicKey: /path/to/public.pem
```

### Token Refresh

Clients should refresh tokens before expiration:

```javascript
client.onTokenExpiring((expiresIn) => {
  if (expiresIn < 60000) {  // < 1 minute
    const newToken = await fetchNewToken();
    client.refreshToken(newToken);
  }
});
```

## See Also

- [Enable TLS](../how-to/security/enable-tls.md) — How to set up encryption
- [Capability Tokens](../how-to/security/capability-tokens.md) — Token configuration
- [Pairing](../how-to/security/pairing.md) — Zero-config setup
