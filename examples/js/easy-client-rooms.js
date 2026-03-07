/**
 * Easy Client SDK -- encrypted rooms example
 *
 * Demonstrates E2E encrypted rooms, password protection,
 * key rotation, and TOFU verification using @clasp-to/sdk.
 *
 * Usage:
 *   npm install @clasp-to/sdk
 *   node easy-client-rooms.js
 */

import clasp from '@clasp-to/sdk'

const CLASP_URL = process.env.CLASP_URL || 'ws://localhost:7330'

// --- Two clients simulating Alice and Bob -----------------------------------

const alice = await clasp(CLASP_URL, { name: 'Alice' })
const bob = await clasp(CLASP_URL, { name: 'Bob' })
console.log('Alice and Bob connected')

// --- Basic encrypted room ---------------------------------------------------

console.log('\n--- Basic Encrypted Room ---')

const aliceRoom = await alice.room('/chat/private')
const bobRoom = await bob.room('/chat/private')

bobRoom.on('/chat/private/messages/**', (msg) => {
  console.log('Bob received:', msg)
})

// Short delay for subscription to propagate
await new Promise(r => setTimeout(r, 500))

await aliceRoom.set('/chat/private/messages/1', {
  text: 'Hello Bob!',
  from: 'alice',
  ts: Date.now(),
})
console.log('Alice sent encrypted message')

// --- Password-protected room ------------------------------------------------

console.log('\n--- Password-Protected Room ---')

const aliceSecret = await alice.room('/chat/secret', { password: 'shhh' })
const bobSecret = await bob.room('/chat/secret', { password: 'shhh' })

await aliceSecret.set('/chat/secret/data', { classified: true })
console.log('Password-protected message sent')

// --- Key rotation -----------------------------------------------------------

console.log('\n--- Key Rotation ---')

const aliceSecure = await alice.room('/chat/secure', { rotateKeys: '1h' })
const bobSecure = await bob.room('/chat/secure', { rotateKeys: '1h' })

bobSecure.on('/chat/secure/messages/**', (msg) => {
  console.log('Bob (secure):', msg)
})

await new Promise(r => setTimeout(r, 500))

await aliceSecure.set('/chat/secure/messages/1', { text: 'Rotating keys!' })

// Manual key rotation
await aliceSecure.rotateKey()
console.log('Key rotated')

await aliceSecure.set('/chat/secure/messages/2', { text: 'After rotation' })

// --- TOFU verification ------------------------------------------------------

console.log('\n--- TOFU Verification ---')

const aliceVerified = await alice.room('/chat/verified', {
  onKeyChange: (peerId, oldFingerprint, newFingerprint) => {
    console.log(`Key changed for ${peerId}`)
    console.log(`  Old: ${oldFingerprint}`)
    console.log(`  New: ${newFingerprint}`)
    return true // accept the new key
  },
})

const bobVerified = await bob.room('/chat/verified', {
  onKeyChange: (peerId, oldFingerprint, newFingerprint) => {
    console.log(`Bob: key changed for ${peerId}`)
    return true
  },
})

await aliceVerified.set('/chat/verified/hello', { text: 'TOFU active' })
console.log('TOFU room message sent')

// --- Cleanup ----------------------------------------------------------------

await new Promise(r => setTimeout(r, 1000))

alice.destroyRoom('/chat/private')
alice.destroyRoom('/chat/secret')
alice.destroyRoom('/chat/secure')
alice.destroyRoom('/chat/verified')

alice.close()
bob.close()
console.log('\nDone')
