# clasp-crypto

[![crates.io](https://img.shields.io/crates/v/clasp-crypto.svg)](https://crates.io/crates/clasp-crypto)
[![docs.rs](https://docs.rs/clasp-crypto/badge.svg)](https://docs.rs/clasp-crypto)

E2E encryption add-on for the [CLASP](https://clasp.to) protocol. Provides client-side encryption that is transparent to the router -- the router never holds keys or decrypts data.

## Features

- **AES-256-GCM** symmetric encryption for signal values
- **ECDH P-256** key agreement for secure key exchange
- **HKDF-SHA256** key derivation
- **ECDSA P-256** digital signatures
- **TOFU** (Trust On First Use) peer key verification
- **Automatic key rotation** with configurable interval (minimum 60s)
- **Replay protection** via nonce tracking
- **Timestamp validation** for stale announcement rejection
- **FileSystemKeyStore** for persistent key storage (behind `fs-store` feature)
- **Cross-platform interop** with `@clasp-to/crypto` (JS) via JWK format

## Quick Start

```rust
use clasp_crypto::{E2ESession, E2ESessionConfig, MemoryKeyStore};
use std::sync::Arc;
use std::time::Duration;

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

// Encrypt
let envelope = session.encrypt(r#"{"fader": 0.75}"#)?;

// Decrypt
let plaintext = session.decrypt(&envelope).await?;
```

## Feature Flags

| Flag | Description |
|------|-------------|
| `client` | `CryptoClient` wrapper for transparent encrypt/decrypt over `clasp-client` |
| `fs-store` | `FileSystemKeyStore` for persistent key storage (requires tokio) |

## Documentation

See the [E2E Encryption Guide](https://clasp.to/docs/auth/e2e-encryption) for the full protocol description, key exchange flow, and security properties.

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT license](../../LICENSE-MIT) at your option.
