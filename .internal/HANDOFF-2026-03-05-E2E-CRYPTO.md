# Handoff - 2026-03-05: E2E Encryption Drop-In Adoption

## Overview

Implemented the CLASP E2E Encryption Drop-In Adoption plan: automatic key rotation (Rust poll-based + JS timer-based), FileSystemKeyStore, timestamp validation, replay protection, integration tests, examples, docs, and CI. Goal: make encryption a 5-line drop-in for any CLASP app.

**Status: All changes uncommitted on branch `update-bridge-app-1`.** 47 Rust tests pass (52 with fs-store), 51 JS tests pass, 4 interop scenarios pass, clippy/fmt clean.

---

## Modified Files

### `crates/clasp-crypto/src/protocol.rs` — Rotation, Timestamp, Replay

Core E2ESession state machine changes:

- **Config additions**: `rotation_interval: Option<Duration>`, `on_rotation: Option<Arc<dyn Fn() + Send + Sync>>`, `max_announcement_age: Option<Duration>`
- **Session state**: `last_rotation: Option<u64>`, `rotation_count: u64`, `seen_nonces: HashSet<String>`
- **New methods**: `should_rotate()`, `maybe_rotate()`, `rotation_count()`, `last_rotation()`
- `enable_encryption()` sets `last_rotation = now_ms()`
- `start()` restores `last_rotation` from `data.stored_at` for restart persistence
- `rotate_key()` increments `rotation_count`, updates `last_rotation`
- `handle_peer_pubkey()` validates announcement timestamp against `max_announcement_age` and rejects >30s future
- `handle_key_exchange()` tracks `"{from_id}:{iv}"` nonces, rejects duplicates, caps HashSet at 10K (full clear)
- `destroy()` clears `seen_nonces`
- Minimum rotation interval enforced: 60 seconds
- 6 new unit tests covering rotation, timestamp validation, replay protection, persistence

### `crates/clasp-crypto/src/client.rs` — CryptoClient Rotation Driving

- `CryptoClientConfig` gained `rotation_interval: Option<Duration>`
- `session()` passes rotation interval from config to new sessions
- **Breaking API**: `set_encrypted`/`emit_encrypted` changed from `(&self, address, &E2ESession, value)` to `(&mut self, address, session_path: &str, value)` — now looks up session internally and drives rotation before encrypting
- New `tick_rotations()` for callers using `tokio::select!` loops
- New private `drive_rotation()` — calls `maybe_rotate()`, publishes announcement, distributes keyex messages

### `crates/clasp-crypto/src/storage.rs` — FileSystemKeyStore + Tests

- New `FileSystemKeyStore` behind `#[cfg(feature = "fs-store")]`
- Layout: `<base_dir>/group-keys/<sha256(session_id)>.json`, `<base_dir>/tofu/<sha256(id)>.json`
- Atomic writes via temp file + rename
- Uses `tokio::fs`, requires `sha2` crate
- 5 new fs-store tests: round-trip, missing key, delete, TOFU, persist-and-reload (simulates restart)

### `crates/clasp-crypto/Cargo.toml`

- Added feature: `fs-store = ["dep:tokio"]`
- Added dependency: `tokio = { workspace = true, features = ["fs"], optional = true }`
- Added dev-dependency: `tempfile = "3.10"` (for fs-store tests)

### `crates/clasp-crypto/src/lib.rs`

- Added `pub use protocol::E2ESessionConfig`
- Added `#[cfg(feature = "fs-store")] pub use storage::FileSystemKeyStore`

### `bindings/js/packages/clasp-crypto/src/types.ts`

- Added to `E2EConfig`: `rotationInterval?: number`, `onRotation?: () => void`, `maxAnnouncementAge?: number`

### `bindings/js/packages/clasp-crypto/src/protocol.ts`

- Timer-based rotation: `setInterval` in `start()` (min 60s enforced), cleared in `destroy()`
- Fields: `rotationTimer`, `_lastRotation`, `_rotationCount`, `seenNonces`
- `start()` restores `_lastRotation` from stored key's `storedAt` (only if key loads successfully)
- Pubkey subscription: timestamp validation (age + future)
- Keyex subscription: nonce tracking with `seenNonces` Set, 10K cap

### `bindings/js/packages/clasp-crypto/package.json`

- Fixed `"test": "vitest"` to `"test": "vitest run"` — prevents CI hang from watch mode

### `docs/concepts/security-model.md`

- Added "End-to-End Encryption" section
- Updated threat model table with E2E mitigations (eavesdropping, replay, compromised router)

### `.github/workflows/ci.yml`

- Added `crypto-interop` job: ubuntu-latest, Rust + Node.js 20
- Runs `cargo test -p clasp-crypto`, `cargo test -p clasp-crypto --features fs-store`, `crypto-interop-tests` binary, JS `npm test`

---

## New Files

### `clasp-e2e/src/bin/crypto_interop_tests.rs`

4 Rust-to-Rust test scenarios (exercises E2ESession directly, no router needed):
1. Key exchange: Alice enables, Bob requests, Bob decrypts
2. Key rotation: Alice rotates, Bob gets new key, old messages fail
3. Multi-peer: 3 clients, all decrypt after joining
4. TOFU violation: key change rejected without callback

### `bindings/js/packages/clasp-crypto/tests/interop.test.ts`

5 JS integration tests:
1. Full JS-to-JS key exchange + encrypt/decrypt
2. Auto-rotation timer fires, peer still decrypts
3. Multi-peer join/leave
4. Timestamp validation rejects stale announcements
5. Replay protection rejects duplicate key exchange

### `examples/js/encrypted-signals.js`

Minimal JS example: CryptoClient with auto-rotation, subscribe + decrypt.

### `examples/rust/encrypted-client.rs`

Illustrative Rust example: E2ESession with FileSystemKeyStore. Note: references `dirs` crate which is not in workspace deps — file is non-compilable, for documentation only.

### `docs/auth/e2e-encryption.md`

Full guide: quick start (JS + Rust), key exchange diagram, rotation, TOFU verification, storage backends, password-gated groups, security properties, cross-platform interop, troubleshooting.

### `clasp-e2e/Cargo.toml` (modified)

- Added `clasp-crypto = { workspace = true, features = ["client"] }`
- Added `[[bin]] name = "crypto-interop-tests"`

---

## Pre-existing Code (not modified by this session)

- **JS `CryptoClient`** (`bindings/js/packages/clasp-crypto/src/client.ts`): Already existed with `set()`, `emit()`, `subscribe()` (transparent encrypt/decrypt), `session()`, `findSession()`, `close()`. Exported from `index.ts`. No changes needed.

## Not Implemented

- **Phase 4c** (cross-platform Rust<>JS interop test): Requires spawning Node child process from Rust — complex, lower ROI given both sides independently tested
- **Phase 6** (relay `_e2e` path debug logging): Low priority per plan

## Known Issues

1. **Breaking Rust CryptoClient API**: `set_encrypted`/`emit_encrypted` signature changed. Existing callers will break at compile time. Intentional — enables internal rotation driving.
2. **`seen_nonces` 10K cap**: Full clear when limit hit, not LRU. Adequate for replay protection threat model.

## Verification

```bash
# Rust unit tests (47 pass)
cargo test -p clasp-crypto

# Rust unit tests with fs-store (52 pass)
cargo test -p clasp-crypto --features fs-store

# Rust interop scenarios (4 pass)
cargo run -p clasp-e2e --bin crypto-interop-tests

# JS tests (51 pass)
cd bindings/js/packages/clasp-crypto && npm test

# Lint
cargo fmt --all -- --check
cargo clippy -p clasp-crypto -- -D warnings
```
