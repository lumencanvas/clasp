// E2E Encrypted Signals Example
//
// Demonstrates using @clasp-to/crypto for end-to-end encryption
// over CLASP signals. The router never sees plaintext.
//
// Usage:
//   node encrypted-signals.js [relay-url]

import { Clasp } from '@clasp-to/core'
import { CryptoClient, MemoryKeyStore } from '@clasp-to/crypto'

const RELAY = process.argv[2] || 'ws://localhost:7330'

async function main() {
  const clasp = new Clasp(RELAY)
  const crypto = new CryptoClient(clasp, {
    identityId: `device-${Math.random().toString(36).slice(2, 8)}`,
    store: new MemoryKeyStore(),
  })

  // Create an encrypted session with auto-rotation every hour
  const session = crypto.session('/myapp/signals', {
    rotationInterval: 3_600_000, // 1 hour
  })
  await session.start()
  await session.enableEncryption()

  console.log('Encryption enabled. Publishing encrypted signals...')

  // Subscribe to all signals under the encrypted path.
  // CryptoClient automatically decrypts E2E envelopes.
  crypto.subscribe('/myapp/signals/**', (value, addr) => {
    console.log(`  ${addr} = ${JSON.stringify(value)}`)
  })

  // Publish encrypted values. The router only sees E2E envelopes.
  await crypto.set('/myapp/signals/fader', 0.75)
  await crypto.set('/myapp/signals/button', true)
  await crypto.emit('/myapp/signals/trigger', { note: 60, velocity: 127 })

  console.log('Signals published. Press Ctrl+C to exit.')
}

main().catch(console.error)
