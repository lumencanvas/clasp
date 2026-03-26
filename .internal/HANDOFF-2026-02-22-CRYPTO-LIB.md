# E2E Crypto Library Extraction — 2026-02-22

## What Was Built

Extracted E2E encryption from the chat app into reusable `@clasp-to/crypto` (JS) and `clasp-crypto` (Rust) packages, then hardened through 4 rounds of security audit.

### JS: `@clasp-to/crypto` — `bindings/js/packages/clasp-crypto/`

| File | Purpose |
|------|---------|
| `src/primitives.ts` | AES-256-GCM, ECDH P-256, ECDSA P-256, HKDF-SHA256, fingerprinting |
| `src/protocol.ts` | `E2ESession` class — key exchange, TOFU, encrypt/decrypt over CLASP paths |
| `src/client.ts` | `CryptoClient` wrapper — transparent encrypt/decrypt for `Clasp` instances |
| `src/storage.ts` | `KeyStore` interface, `MemoryKeyStore`, `IndexedDBKeyStore` |
| `src/types.ts` | `E2EEnvelope`, `E2EConfig`, `KeyData`, `TofuRecord`, etc. |
| `tests/primitives.test.ts` | 25 tests — crypto primitives |
| `tests/protocol.test.ts` | 21 tests — E2ESession protocol |

**46 JS tests passing.** Zero runtime deps (Web Crypto API built-in). Peer dep on `@clasp-to/core`.

### Rust: `clasp-crypto` — `crates/clasp-crypto/`

| File | Purpose |
|------|---------|
| `src/primitives.rs` | AES-256-GCM (aes-gcm), ECDH P-256 (p256), HKDF (hkdf), ECDSA, JWK interop |
| `src/protocol.rs` | `E2ESession` state machine — mirrors JS protocol |
| `src/client.rs` | `CryptoClient` wrapper (behind `client` feature flag) |
| `src/storage.rs` | `KeyStore` trait, `MemoryKeyStore` |
| `src/types.rs` | `E2EEnvelope`, `ECDHKeyPair`, `KeyExchangeMessage`, etc. |
| `src/error.rs` | `CryptoError` enum |

**40 Rust tests passing.** Deps: `aes-gcm`, `p256`, `hkdf`, `sha2`, `zeroize`, `serde_json`, `base64`.

## Architecture

### Two Layers

1. **Primitives** (no CLASP dependency) — pure crypto operations
2. **Protocol** (depends on CLASP paths) — key exchange over `{basePath}/_e2e/` subpaths

### Key Design Decisions

- **Separate package** — `@clasp-to/core` stays lean, crypto is opt-in
- **Wrapper pattern** — `CryptoClient` wraps `Clasp` with same API + transparent encrypt/decrypt
- **Value-level encryption** — envelope `{ _e2e: 1, ct, iv, v }` flows as normal CLASP map value
- **JWK interop** — JS and Rust use JWK format for cross-platform key serialization
- **Fingerprint normalization** — `{crv, kty, x, y}` canonical JSON, SHA-256, matching across platforms

### Wire Format

- Key exchange paths: `{basePath}/_e2e/pubkey/{id}`, `{basePath}/_e2e/keyex/{targetId}`
- Envelope: `{ _e2e: 1, ct: base64, iv: base64, v: 1 }`
- HKDF info: `clasp-e2e-keyex-v1`, salt: 32 zero bytes

## Security Audit History (4 Rounds)

### Round 1 — Initial
- TOFU enforcement (callback must return `=== true`)
- Fingerprint normalization to `{crv, kty, x, y}` for EC keys
- `constantTimeEqual` for fingerprint/password comparisons
- Derived key made non-extractable

### Round 2
- Empty `fromId` rejection in keyex handler (TOFU bypass fix)
- `Uint8Array.fill(0)` zeroization of plaintext key material in JS
- `Zeroizing<>` wrappers throughout Rust protocol
- `Zeroize + ZeroizeOnDrop` on `ECDHKeyPair` and `SigningKeyPair`
- Destroyed checks on encrypt/decrypt
- Timer cleanup in `waitForPasswordProof`
- Defensive copies in `MemoryKeyStore`
- Mutex poisoning handling in Rust `MemoryKeyStore`

### Round 3
- Race condition in JS `getECDHKeyPair` (promise-based lock)
- Envelope version check in decrypt (`v !== 1` / `v != 1`)
- Rust `Drop` impl for `E2ESession`
- Rust `ensure_ecdh_key_pair` + `ecdh_key_pair` pattern (no private key clone)
- Removed `Clone` from `ECDHKeyPair` and `SigningKeyPair`
- `canonical_json` key escaping via `serde_json::to_string`
- Destroyed guards on `handle_peer_pubkey` and `handle_key_exchange`
- Dead `password_hash` code removed from Rust (documented as caller responsibility)

### Round 4
- TOFU `firstSeen` preserved on accepted key change (both JS and Rust)
- Reject keyex when group key already exists (prevents rogue peer key replacement)
- `destroyed` checks after every `await` in JS subscribe handlers
- `destroyed` check in `distributeKeyToPeers` loop
- JWK type validation: `jwk_to_public_key` requires `kty=EC, crv=P-256`; `jwk_to_group_key` requires `kty=oct`
- Cache sender's public key in Rust `handle_key_exchange` for future rotations
- `_e2e` marker check in Rust `decrypt`
- Strict `=== true` on JS `onKeyChange` callback result

## Known Design Limitations (Accepted)

| Limitation | Rationale |
|------------|-----------|
| ECDH key pair not rotated on `rotateKey()` | No forward secrecy across rotations; would require TOFU re-handshake |
| `fromId` self-asserted in keyex | TOFU alone can't prevent relay-level MITM; would need signing |
| `constantTimeEqual` leaks length | Acceptable — fingerprints/hashes are always fixed length |
| JS strings/objects can't be zeroized | Platform limitation; `Uint8Array` buffers are zeroized |
| `password_hash` not enforced in Rust | Documented as caller responsibility; JS handles via subscription |
| `CryptoClient.session()` ignores options on 2nd call | Returns cached session |
| `peer_public_keys` cache unbounded | Caller/auth layer responsibility |
| No `CryptoClient` test coverage | Covered by E2ESession tests; client is thin wrapper |

## Test Results

```
JS:   46 tests PASS (25 primitives + 21 protocol)
Rust: 40 tests PASS (20 primitives + 4 storage + 16 protocol)
```

## How to Build/Test

```bash
# JS
cd bindings/js/packages/clasp-crypto
npm run build         # tsup -> CJS + ESM + DTS
npx vitest run        # 46 tests

# Rust
cargo build -p clasp-crypto
cargo test -p clasp-crypto   # 40 tests
```

## Usage Example

```typescript
import { ClaspBuilder } from '@clasp-to/core'
import { CryptoClient, IndexedDBKeyStore } from '@clasp-to/crypto'

const clasp = await new ClaspBuilder('wss://relay.clasp.to').withName('Alice').connect()
const crypto = new CryptoClient(clasp, {
  identityId: 'alice-uuid',
  store: new IndexedDBKeyStore('my-app'),
  onKeyChange: (peerId, oldFp, newFp) => confirm(`${peerId} key changed!`),
})

const session = crypto.session('/myapp/room/general')
await session.start()
await session.enableEncryption()

// Send encrypted
crypto.emit('/myapp/room/general/messages', { text: 'hello' })

// Receive decrypted
crypto.subscribe('/myapp/room/general/messages', (data) => {
  console.log(data.text) // 'hello'
})
```

## Files Modified

| File | Change |
|------|--------|
| `Cargo.toml` (root) | Added `crates/clasp-crypto` to workspace members |

## Files Created

All files under `bindings/js/packages/clasp-crypto/` and `crates/clasp-crypto/` are new.

## Chat App Migration (Deferred)

The chat app (`apps/chat/`) still uses its own `useCrypto.js` composable. Migration to `@clasp-to/crypto` is a separate task. The library API was designed to be a drop-in replacement — `E2ESession` generalizes the chat app's key exchange protocol with a framework-agnostic API.
