---
title: Unified Identity
description: One Ed25519 keypair for CLASP, DID, and libp2p
order: 7
---

# Unified Identity

The `clasp-identity` crate produces three interoperable identity formats from a single Ed25519 keypair:

| Format | Example | Used By |
|--------|---------|---------|
| CLASP EntityId | `clasp:3vQB7B6mGG...` | clasp-registry, clasp-caps (auth tokens) |
| W3C DID | `did:key:z6MkhaXg...` | DefraDB ACP (access control policies) |
| libp2p PeerID | `12D3KooWPvQ8cn...` | DefraDB P2P networking |

One key, three systems.

## Usage

```rust
use clasp_identity::Identity;

let id = Identity::generate();

// All three derive from the same 32-byte Ed25519 public key
println!("EntityId: {}", id.entity_id());  // clasp:3vQB7B...
println!("DID:      {}", id.did());         // did:key:z6Mk...
println!("PeerID:   {}", id.peer_id());     // 12D3KooW...

// Sign and verify
let sig = id.sign(b"hello world");
assert!(id.verify(b"hello world", &sig));

// Create from existing key
let id2 = Identity::from_bytes(&key_bytes)?;
```

## Encoding Details

**DID (did:key method)**:
1. Take 32-byte Ed25519 public key
2. Prepend multicodec varint `0xed 0x01` (Ed25519)
3. Base58btc encode
4. Prepend `did:key:z`

**libp2p PeerID**:
1. Take 32-byte Ed25519 public key
2. Wrap in protobuf: `0x08 0x01 0x12 0x20` + 32 bytes (KeyType=Ed25519, Data=key)
3. Since 36 bytes <= 42, use identity multihash: `0x00 0x24` + 36 bytes
4. Base58btc encode

**CLASP EntityId**:
1. Take first 16 bytes of the 32-byte public key
2. Base58 encode
3. Prepend `clasp:`

## No libp2p Dependency

PeerID encoding is implemented manually to avoid pulling in the full libp2p dependency tree. The crate depends only on `ed25519-dalek`, `bs58`, `sha2`, and `unsigned-varint`.
