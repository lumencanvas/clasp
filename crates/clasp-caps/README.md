# clasp-caps

Delegatable capability tokens with Ed25519 signatures for CLASP.

## Features

- **Ed25519 Signatures** - Tokens are cryptographically signed, no shared secrets needed
- **Delegation Chains** - Delegate tokens with attenuated scopes (UCAN-style)
- **Scope Attenuation** - Child tokens can only narrow permissions, never widen them
- **Expiration Clamping** - Child tokens cannot outlive their parent
- **Chain Depth Limits** - Configurable maximum delegation depth
- **ValidatorChain Integration** - Works alongside CPSK and Entity validators

## Installation

```toml
[dependencies]
clasp-caps = "3.5"
```

## Usage

### Create a Root Token

```rust
use clasp_caps::CapabilityToken;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;

// Generate a root keypair
let root_key = SigningKey::generate(&mut OsRng);

// Create a root token with admin access, valid for 24 hours
let expires = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs() + 86400;

let root_token = CapabilityToken::create_root(
    &root_key,
    vec!["admin:/**".to_string()],
    expires,
    None, // bearer token (no audience restriction)
)?;

// Encode for transmission
let wire = root_token.encode()?; // "cap_<base64url(msgpack)>"
```

### Delegate a Token

```rust
// Generate a child keypair
let child_key = SigningKey::generate(&mut OsRng);

// Delegate with narrower scopes -- child can only write to /lights/**
let child_token = root_token.delegate(
    &child_key,
    vec!["write:/lights/**".to_string()],
    expires,
    None,
)?;

// Delegate again with even narrower scopes
let grandchild_key = SigningKey::generate(&mut OsRng);
let grandchild_token = child_token.delegate(
    &grandchild_key,
    vec!["read:/lights/room1/**".to_string()],
    expires,
    None,
)?;
```

### Decode and Inspect

```rust
let decoded = CapabilityToken::decode(&wire)?;
assert_eq!(decoded.chain_depth(), 0); // root token
assert!(!decoded.is_expired());
decoded.verify_signature()?;
```

### Validate with Trust Anchors

```rust
use clasp_caps::CapabilityValidator;

let root_pubkey = root_key.verifying_key().to_bytes().to_vec();

let validator = CapabilityValidator::new(
    vec![root_pubkey], // trust anchors
    5,                 // max chain depth
);

// Implements clasp_core::security::TokenValidator
use clasp_core::security::TokenValidator;
match validator.validate(&wire) {
    clasp_core::security::ValidationResult::Valid(info) => {
        println!("Scopes: {:?}", info.scopes);
    }
    other => println!("Validation failed: {:?}", other),
}
```

## Token Wire Format

Tokens use the `cap_` prefix followed by URL-safe base64-encoded MessagePack:

```
cap_<base64url(msgpack(CapabilityToken))>
```

## Delegation Chain

```
  Root Token (issuer: KeyA, scopes: admin:/**)
       │
       │  delegate() -- scope attenuation enforced
       ▼
  Child Token (issuer: KeyB, scopes: write:/lights/**)
       │  proofs: [ProofLink{issuer: KeyA, scopes, sig}]
       │
       │  delegate() -- expiration clamped to parent
       ▼
  Grandchild (issuer: KeyC, scopes: read:/lights/room1/**)
       proofs: [ProofLink{KeyA,...}, ProofLink{KeyB,...}]
```

Each delegation step enforces:
1. **Scope subset** -- child scopes must be a subset of parent scopes
2. **Action hierarchy** -- `admin > write > read` (custom actions require exact match)
3. **Pattern subset** -- child patterns must be covered by parent patterns
4. **Expiration clamping** -- `child_expires = min(child_expires, parent.expires_at)`

## Configuration Reference

### CapabilityToken

| Field | Type | Description |
|-------|------|-------------|
| `version` | `u8` | Token version (currently `1`) |
| `issuer` | `Vec<u8>` | Issuer's Ed25519 public key (32 bytes) |
| `audience` | `Option<Vec<u8>>` | Audience public key (`None` = bearer token) |
| `scopes` | `Vec<String>` | Scopes in `action:pattern` format |
| `expires_at` | `u64` | Expiration as Unix timestamp (seconds) |
| `nonce` | `String` | UUID v4 nonce for replay prevention |
| `proofs` | `Vec<ProofLink>` | Delegation chain (empty for root tokens) |
| `signature` | `Vec<u8>` | Ed25519 signature (64 bytes) |

### CapabilityValidator

| Parameter | Type | Description |
|-----------|------|-------------|
| `trust_anchors` | `Vec<Vec<u8>>` | Trusted root issuer public keys (32 bytes each) |
| `max_depth` | `usize` | Maximum allowed delegation chain depth |

### Action Hierarchy

| Parent Action | Allowed Child Actions |
|---------------|----------------------|
| `admin` | `admin`, `write`, `read`, any custom |
| `write` | `write`, `read` |
| `read` | `read` only |
| custom | exact match only |

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

---

Maintained by [LumenCanvas](https://lumencanvas.studio)
