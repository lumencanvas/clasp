# clasp-identity

Unified Ed25519 identity for CLASP, DID, and libp2p. One keypair produces three interoperable identity formats:

- **CLASP EntityId**: `clasp:<base58>` (compatible with clasp-registry)
- **DID**: `did:key:z6Mk...` (W3C Decentralized Identifier, Ed25519 multicodec)
- **libp2p PeerID**: `12D3KooW...` (protobuf-encoded, identity multihash)

## Usage

```rust
use clasp_identity::Identity;

let id = Identity::generate();

println!("EntityId: {}", id.entity_id());
println!("DID:      {}", id.did());
println!("PeerID:   {}", id.peer_id());

// Sign and verify
let sig = id.sign(b"hello");
assert!(id.verify(b"hello", &sig));
```

## Why

CLASP uses Ed25519 for entity identity. DefraDB uses Ed25519 for peer identity and DIDs for access control. This crate bridges all three so a single keypair works across both systems.

## No external dependencies on libp2p

PeerID encoding is implemented manually (protobuf + multihash + base58) to avoid pulling in the full libp2p dependency tree.

## License

MIT OR Apache-2.0
