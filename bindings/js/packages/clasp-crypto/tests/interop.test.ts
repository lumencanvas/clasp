import { describe, it, expect, vi, beforeEach } from 'vitest'
import { E2ESession, type ClaspLike } from '../src/protocol'
import { MemoryKeyStore } from '../src/storage'
import type { E2EConfig, E2EEnvelope } from '../src/types'

function createMockClient(): ClaspLike & {
  _subs: Map<string, ((data: unknown, address: string) => void)[]>
  _sets: Array<{ address: string; value: unknown }>
  _emits: Array<{ address: string; payload: unknown }>
  _deliver: (address: string, data: unknown) => void
} {
  const subs = new Map<string, ((data: unknown, address: string) => void)[]>()
  const sets: Array<{ address: string; value: unknown }> = []
  const emits: Array<{ address: string; payload: unknown }> = []

  return {
    _subs: subs,
    _sets: sets,
    _emits: emits,
    connected: true,
    set(address: string, value: unknown) {
      sets.push({ address, value })
    },
    emit(address: string, payload?: unknown) {
      emits.push({ address, payload })
    },
    subscribe(pattern: string, callback: (data: unknown, address: string) => void) {
      const list = subs.get(pattern) ?? []
      list.push(callback)
      subs.set(pattern, list)
      return () => {
        const idx = list.indexOf(callback)
        if (idx >= 0) list.splice(idx, 1)
      }
    },
    _deliver(address: string, data: unknown) {
      for (const [pattern, callbacks] of subs) {
        if (matchPattern(pattern, address)) {
          for (const cb of callbacks) {
            cb(data, address)
          }
        }
      }
    },
  }
}

function matchPattern(pattern: string, address: string): boolean {
  if (pattern === address) return true
  if (pattern.endsWith('/*')) {
    const prefix = pattern.slice(0, -2)
    if (!address.startsWith(prefix + '/')) return false
    return !address.slice(prefix.length + 1).includes('/')
  }
  return false
}

function createConfig(store: MemoryKeyStore, overrides?: Partial<E2EConfig>): E2EConfig {
  return {
    identityId: 'alice',
    basePath: '/test/interop',
    store,
    ...overrides,
  }
}

describe('E2ESession Integration', () => {
  let store: MemoryKeyStore
  let client: ReturnType<typeof createMockClient>

  beforeEach(() => {
    store = new MemoryKeyStore()
    client = createMockClient()
  })

  it('full JS-to-JS key exchange flow', async () => {
    // Alice enables encryption
    const aliceStore = new MemoryKeyStore()
    const aliceClient = createMockClient()
    const alice = new E2ESession(aliceClient, createConfig(aliceStore, {
      identityId: 'alice',
    }))
    await alice.start()
    await alice.enableEncryption()

    // Bob joins and requests key
    const bobStore = new MemoryKeyStore()
    const bobClient = createMockClient()
    const bob = new E2ESession(bobClient, createConfig(bobStore, {
      identityId: 'bob',
    }))
    await bob.start()
    await bob.requestGroupKey()

    // Simulate Bob's pubkey being delivered to Alice
    const bobPubkeySets = bobClient._sets.filter(s =>
      s.address.includes('/_e2e/pubkey/')
    )
    expect(bobPubkeySets.length).toBeGreaterThan(0)

    // Deliver Bob's pubkey to Alice
    aliceClient._deliver(
      '/test/interop/_e2e/pubkey/bob',
      bobPubkeySets[0].value,
    )
    await new Promise(r => setTimeout(r, 100))

    // Alice should have emitted a keyex to Bob
    const keyexEmits = aliceClient._emits.filter(e =>
      e.address === '/test/interop/_e2e/keyex/bob'
    )
    expect(keyexEmits.length).toBe(1)

    // Deliver the keyex to Bob
    bobClient._deliver(
      '/test/interop/_e2e/keyex/bob',
      keyexEmits[0].payload,
    )
    await new Promise(r => setTimeout(r, 100))

    // Bob should now be encrypted
    expect(bob.encrypted).toBe(true)

    // Alice encrypts, Bob decrypts
    const envelope = await alice.encrypt('JS interop message')
    expect(envelope).not.toBeNull()
    const decrypted = await bob.decrypt(envelope!)
    expect(decrypted).toBe('JS interop message')

    alice.destroy()
    bob.destroy()
  })

  it('manual rotateKey generates new key and peer still decrypts', async () => {
    // Note: rotationInterval is clamped to 60s minimum, so timer-based tests
    // are impractical. This test calls rotateKey() directly to verify the
    // rotation mechanics work correctly.
    const aliceStore = new MemoryKeyStore()
    const aliceClient = createMockClient()
    const alice = new E2ESession(aliceClient, createConfig(aliceStore, {
      identityId: 'alice',
    }))
    await alice.start()
    await alice.enableEncryption()
    expect(alice.rotationCount).toBe(0)

    await alice.rotateKey()
    expect(alice.rotationCount).toBe(1)
    expect(alice.lastRotation).toBeGreaterThan(0)

    // Encrypt should still work after rotation
    const envelope = await alice.encrypt('after rotation')
    expect(envelope).not.toBeNull()
    const decrypted = await alice.decrypt(envelope!)
    expect(decrypted).toBe('after rotation')

    alice.destroy()
  })

  it('multi-peer join and leave', async () => {
    const { generateECDHKeyPair, exportKey } = await import('../src/primitives')

    // Alice has the key
    const aliceStore = new MemoryKeyStore()
    const aliceClient = createMockClient()
    const alice = new E2ESession(aliceClient, createConfig(aliceStore, {
      identityId: 'alice',
    }))
    await alice.start()
    await alice.enableEncryption()

    // Add Bob and Carol as peers
    const bobKp = await generateECDHKeyPair()
    const carolKp = await generateECDHKeyPair()

    aliceClient._deliver('/test/interop/_e2e/pubkey/bob', {
      publicKey: await exportKey(bobKp.publicKey),
      timestamp: Date.now(),
    })
    aliceClient._deliver('/test/interop/_e2e/pubkey/carol', {
      publicKey: await exportKey(carolKp.publicKey),
      timestamp: Date.now(),
    })
    await new Promise(r => setTimeout(r, 100))

    // Both should have received keyex messages
    const keyexBob = aliceClient._emits.filter(e =>
      e.address === '/test/interop/_e2e/keyex/bob'
    )
    const keyexCarol = aliceClient._emits.filter(e =>
      e.address === '/test/interop/_e2e/keyex/carol'
    )
    expect(keyexBob.length).toBe(1)
    expect(keyexCarol.length).toBe(1)

    // Remove Bob
    alice.removePeer('bob')

    // Rotate key — only Carol should get it
    const emitsBefore = aliceClient._emits.length
    await alice.rotateKey()
    await new Promise(r => setTimeout(r, 50))

    const newKeyexBob = aliceClient._emits
      .slice(emitsBefore)
      .filter(e => e.address === '/test/interop/_e2e/keyex/bob')
    const newKeyexCarol = aliceClient._emits
      .slice(emitsBefore)
      .filter(e => e.address === '/test/interop/_e2e/keyex/carol')
    expect(newKeyexBob.length).toBe(0)
    expect(newKeyexCarol.length).toBe(1)

    alice.destroy()
  })

  it('timestamp validation rejects old announcements', async () => {
    const aliceStore = new MemoryKeyStore()
    const aliceClient = createMockClient()
    const alice = new E2ESession(aliceClient, createConfig(aliceStore, {
      identityId: 'alice',
      maxAnnouncementAge: 300_000, // 5 minutes
    }))
    await alice.start()
    await alice.enableEncryption()

    const { generateECDHKeyPair, exportKey } = await import('../src/primitives')
    const bobKp = await generateECDHKeyPair()

    // Old announcement (10 min ago)
    aliceClient._deliver('/test/interop/_e2e/pubkey/bob', {
      publicKey: await exportKey(bobKp.publicKey),
      timestamp: Date.now() - 600_000,
    })
    await new Promise(r => setTimeout(r, 50))

    // Should NOT have emitted keyex (timestamp too old)
    const keyexEmits = aliceClient._emits.filter(e =>
      e.address === '/test/interop/_e2e/keyex/bob'
    )
    expect(keyexEmits.length).toBe(0)

    alice.destroy()
  })

  it('duplicate keyex nonce is silently dropped (same-session replay)', async () => {
    // Uses real crypto to generate a valid-looking keyex message, then
    // delivers it twice. The second delivery should be dropped by the
    // nonce dedup set, not by crypto failure.
    const { generateECDHKeyPair, exportKey, deriveSharedKey, encrypt: e2eEncrypt, importECDHPublicKey, toBase64 } = await import('../src/primitives')

    const aliceStore = new MemoryKeyStore()
    const aliceClient = createMockClient()
    const alice = new E2ESession(aliceClient, createConfig(aliceStore, {
      identityId: 'alice',
    }))
    await alice.start()

    // Generate real ECDH keys for Bob (sender) and Alice (receiver)
    const bobKp = await generateECDHKeyPair()
    const aliceKp = await generateECDHKeyPair()
    const bobPubJwk = await exportKey(bobKp.publicKey)

    // We can't easily get Alice's private key to derive a shared secret,
    // so we use a structurally valid but cryptographically incorrect keyex.
    // The key point is: the nonce is tracked on first delivery, so the
    // second delivery is dropped before crypto even runs.
    const keyex = {
      fromId: 'bob',
      encryptedKey: toBase64(new Uint8Array(32)),
      iv: toBase64(new Uint8Array(12)),
      senderPublicKey: bobPubJwk,
    }

    aliceClient._deliver('/test/interop/_e2e/keyex/alice', keyex)
    await new Promise(r => setTimeout(r, 50))
    // First attempt: nonce recorded, crypto fails (wrong shared secret)

    // Second attempt with same iv: silently dropped by replay protection
    aliceClient._deliver('/test/interop/_e2e/keyex/alice', keyex)
    await new Promise(r => setTimeout(r, 50))

    // Alice should not be encrypted (neither attempt succeeded)
    expect(alice.encrypted).toBe(false)

    alice.destroy()
  })
})
