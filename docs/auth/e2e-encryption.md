---
title: E2E Encryption
description: End-to-end encryption for CLASP signals — router-transparent, no server trust required
order: 6
---

# E2E Encryption

CLASP's E2E encryption module (`clasp-crypto` / `@clasp-to/crypto`) provides client-side encryption that is transparent to the router. The router never holds keys and never decrypts data. This is a separate layer from TLS transport encryption and token-based authentication.

## Quick Start

### JavaScript (5 lines)

```js
import { Clasp } from '@clasp-to/core'
import { CryptoClient, MemoryKeyStore } from '@clasp-to/crypto'

const clasp = new Clasp('ws://localhost:7330')
const crypto = new CryptoClient(clasp, {
  identityId: 'device-1',
  store: new MemoryKeyStore(),
})
const session = crypto.session('/myapp/signals', { rotationInterval: 3600000 })
await session.start()
await session.enableEncryption()
```

### Rust (5 lines)

```rust
use clasp_crypto::{E2ESession, E2ESessionConfig, MemoryKeyStore};
use std::sync::Arc;

let store = Arc::new(MemoryKeyStore::new());
let mut session = E2ESession::new(E2ESessionConfig {
    identity_id: "device-1".into(),
    base_path: "/myapp/signals".into(),
    store,
    on_key_change: None,
    password_hash: None,
    rotation_interval: Some(Duration::from_secs(3600)),
    on_rotation: None,
    max_announcement_age: None,
});
session.start().await?;
session.enable_encryption().await?;
```

## How It Works

### Key Exchange Flow

```
Alice (has group key)                   Router                    Bob (joining)
       |                                  |                           |
       | 1. publish ECDH pubkey           |                           |
       |  SET /_e2e/pubkey/alice          |                           |
       |--------------------------------->|                           |
       |                                  |                           |
       |                                  |  2. Bob announces pubkey  |
       |                                  |  SET /_e2e/pubkey/bob     |
       |                                  |<--------------------------|
       |                                  |                           |
       | 3. Alice receives Bob's pubkey   |                           |
       |<---------------------------------|                           |
       |                                  |                           |
       | 4. ECDH shared secret            |                           |
       | 5. Encrypt group key with shared |                           |
       | 6. EMIT /_e2e/keyex/bob          |                           |
       |--------------------------------->|                           |
       |                                  | 7. Deliver to Bob         |
       |                                  |-------------------------->|
       |                                  |                           |
       |                                  |  8. ECDH shared secret    |
       |                                  |  9. Decrypt group key     |
       |                                  |                           |
       |          10. Both encrypt/decrypt with shared group key      |
```

### Cryptographic Primitives

| Primitive | Algorithm | Purpose |
|---|---|---|
| Symmetric encryption | AES-256-GCM | Encrypt/decrypt signal values |
| Key agreement | ECDH P-256 | Derive shared secrets for key exchange |
| Key derivation | HKDF-SHA256 | Derive AES keys from ECDH shared secrets |
| Digital signatures | ECDSA P-256 | Sign/verify (available, not used in key exchange) |
| Fingerprinting | SHA-256 | TOFU key verification |

### Envelope Format

Encrypted values flow through the router as normal CLASP values with the following structure:

```json
{
  "_e2e": 1,
  "ct": "<base64 ciphertext>",
  "iv": "<base64 IV, 12 bytes>",
  "v": 1
}
```

The router treats this as an opaque map value and delivers it to subscribers unchanged.

## Key Rotation

### Automatic Rotation

Set `rotationInterval` (JS, in ms) or `rotation_interval` (Rust, as `Duration`) to enable automatic rotation:

- **JS**: Timer-based via `setInterval`. Fires while the session is active.
- **Rust**: Poll-based via `maybe_rotate()`. Called automatically by `CryptoClient` before each `set_encrypted`/`emit_encrypted`, or manually via `tick_rotations()`.

Minimum interval: 60 seconds (enforced).

### Manual Rotation

Call `rotateKey()` (JS) or `rotate_key()` (Rust) at any time. The new key is distributed to all cached peers immediately.

### What Happens During Rotation

1. A new AES-256-GCM group key is generated.
2. The old key is securely zeroed.
3. The new key is persisted to the key store.
4. A fresh ECDH public key announcement is published.
5. The new group key is encrypted and sent to each cached peer via ECDH.
6. Messages encrypted with the old key can no longer be decrypted.

## TOFU Verification

Trust On First Use: the first public key seen from a peer is stored. If the key changes, the `onKeyChange` callback is invoked:

- If the callback returns `true`, the new key is accepted and the TOFU record is updated.
- If it returns `false` or is absent, the key change is rejected with a `TofuViolation` error.

### UX Guidance

Show a warning when `onKeyChange` fires. Display the old and new fingerprints (hex groups of 4) and let the user decide. For automated systems, consider an out-of-band verification channel.

## Storage Backends

| Backend | Platform | Persistence | Use Case |
|---|---|---|---|
| `MemoryKeyStore` | Any | None (session only) | Testing, ephemeral sessions |
| `IndexedDBKeyStore` | Browser | IndexedDB | Web apps |
| `FileSystemKeyStore` | Rust (fs-store feature) | JSON files | CLI tools, servers, desktop apps |

### FileSystemKeyStore Layout

```
<base_dir>/
  group-keys/
    <sha256(session_id)>.json
  tofu/
    <sha256(peer_id)>.json
```

Uses atomic writes (temp file + rename) for crash safety.

## Password-Gated Groups

Set `passwordHash` in the session config to require peers to prove knowledge of a password before receiving the group key. The peer must publish a proof hash to `/_e2e/proof/<peer_id>` within 2 seconds.

## Timestamp Validation and Replay Protection

### Announcement Timestamps

Set `maxAnnouncementAge` to reject stale public key announcements:
- Announcements older than the configured duration are rejected.
- Announcements more than 30 seconds in the future are rejected.
- Default: disabled (set to `None`/`undefined`). Recommended: 5 minutes (300000 ms).

### Replay Protection

Key exchange messages include a unique IV (nonce). The session tracks seen `{from_id}:{iv}` pairs and rejects duplicates. The nonce set is capped at 10,000 entries to bound memory.

## Security Properties

**What E2E encryption provides:**
- Confidentiality: only participants with the group key can read signal values.
- Integrity: AES-GCM authentication detects tampering.
- Forward secrecy on rotation: old keys are zeroed; compromising a new key does not reveal old messages.

**What E2E encryption does NOT provide:**
- Metadata privacy: the router sees addresses, timing, and message sizes.
- Authentication of the sender: any participant with the group key can encrypt. Use ECDSA signing for sender authentication.
- Protection against a compromised participant: a peer with the group key can share it.
- Deniability: messages can be attributed to the group key holder.

## CryptoClient Wrapper

For convenience, both platforms provide a `CryptoClient` wrapper that handles encryption/decryption transparently:

### JavaScript CryptoClient

```js
import { Clasp } from '@clasp-to/core'
import { CryptoClient, MemoryKeyStore } from '@clasp-to/crypto'

const clasp = new Clasp('ws://localhost:7330')
const crypto = new CryptoClient(clasp, {
  identityId: 'device-1',
  store: new MemoryKeyStore(),
})
const session = crypto.session('/myapp/signals')
await session.start()
await session.enableEncryption()

// Transparent encrypt/decrypt
await crypto.set('/myapp/signals/fader', 0.75)
crypto.subscribe('/myapp/signals/**', (data, addr) => console.log(addr, data))
```

### Rust CryptoClient

```rust
use clasp_crypto::client::{CryptoClient, CryptoClientConfig};
use clasp_crypto::MemoryKeyStore;
use std::sync::Arc;

let client = Clasp::connect_to("ws://localhost:7330").await?;
let store = Arc::new(MemoryKeyStore::new());
let mut crypto = CryptoClient::new(client, CryptoClientConfig {
    identity_id: "device-1".into(),
    store,
    on_key_change: None,
    rotation_interval: None,
});
let session = crypto.session("/myapp/signals");
session.start().await?;
session.enable_encryption().await?;

// Encrypt and send through the inner client
crypto.set_encrypted("/myapp/signals/fader", "/myapp/signals", "0.75").await?;
```

See the [Rust crate README](../../crates/clasp-crypto/README.md) and [JS package README](../../bindings/js/packages/clasp-crypto/README.md) for full API details.

## Cross-Platform Interop

Rust and JS implementations use the same algorithms and wire formats, enabling interoperability:
- Keys are exchanged as JWK JSON (Web Crypto API format).
- Ciphertext and IVs are base64-encoded.
- TOFU fingerprints use canonical JSON + SHA-256 over JWK fields (`{crv, kty, x, y}`).
- HKDF info string: `clasp-e2e-keyex-v1`.
- AES-GCM with 12-byte random IV.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| Decryption returns null/error | Wrong group key (rotation happened) | Request the latest key via `requestGroupKey()` |
| TOFU violation | Peer regenerated their ECDH key pair | Verify out-of-band, then accept via `onKeyChange` callback |
| Key exchange never completes | Peer not connected or wrong base path | Check both peers use the same `basePath` |
| Old messages fail after rotation | Expected behavior | Rotation invalidates old keys by design |
| `maxAnnouncementAge` rejections | Clock skew between peers | Increase the age tolerance or sync clocks |
