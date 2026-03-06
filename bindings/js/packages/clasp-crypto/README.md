# @clasp-to/crypto

[![npm](https://img.shields.io/npm/v/@clasp-to/crypto.svg)](https://www.npmjs.com/package/@clasp-to/crypto)

E2E encryption add-on for [CLASP](https://clasp.to) protocol clients. Provides client-side AES-256-GCM encryption that is transparent to the router.

## Install

```bash
npm install @clasp-to/crypto
```

Peer dependency: `@clasp-to/core ^4.0.0`

## Quick Start

```ts
import { Clasp } from '@clasp-to/core'
import { CryptoClient, MemoryKeyStore } from '@clasp-to/crypto'

const clasp = new Clasp('ws://localhost:7330')
const crypto = new CryptoClient(clasp, {
  identityId: 'device-1',
  store: new MemoryKeyStore(),
})

// Create an encrypted session
const session = crypto.session('/myapp/signals', {
  rotationInterval: 3_600_000, // 1 hour
})
await session.start()
await session.enableEncryption()

// Encrypted set/emit -- CryptoClient handles encryption automatically
await crypto.set('/myapp/signals/fader', 0.75)
await crypto.emit('/myapp/signals/trigger', { note: 60 })

// Encrypted subscribe -- auto-decrypts E2E envelopes
crypto.subscribe('/myapp/signals/**', (data, address) => {
  console.log(address, data) // decrypted automatically
})
```

## API

### CryptoClient

Wraps a `Clasp` instance for transparent encrypt/decrypt.

- `session(basePath, options?)` -- get or create an `E2ESession`
- `set(address, value)` -- encrypts if a session matches, otherwise passes through
- `emit(address, payload?)` -- same behavior as `set()`
- `subscribe(pattern, callback)` -- auto-decrypts E2E envelopes before callback
- `close()` -- destroys all sessions

### E2ESession

Manages key exchange and encryption for one group/room/channel.

- `start()` -- subscribe to key exchange paths, load persisted key
- `enableEncryption()` -- generate a new group key
- `requestGroupKey()` -- request the group key from peers
- `encrypt(value)` -- encrypt a string into an `E2EEnvelope`
- `decrypt(envelope)` -- decrypt an `E2EEnvelope` back to a string
- `rotateKey()` -- generate a new key and distribute to peers
- `destroy()` -- clean up subscriptions and zero key material

### Storage Backends

- `MemoryKeyStore` -- in-memory, for testing or ephemeral sessions
- `IndexedDBKeyStore` -- browser-based persistent storage

## Documentation

See the [E2E Encryption Guide](https://clasp.to/docs/auth/e2e-encryption) for the full protocol description, key exchange flow, and security properties.

## License

MIT
