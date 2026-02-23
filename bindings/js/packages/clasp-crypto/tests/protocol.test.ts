import { describe, it, expect, vi, beforeEach } from 'vitest'
import { E2ESession, type ClaspLike } from '../src/protocol'
import { MemoryKeyStore } from '../src/storage'
import type { E2EConfig, E2EEnvelope } from '../src/types'

/** Create a mock CLASP client for testing. */
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
    /** Deliver a message to all matching subscriptions (exact or wildcard). */
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

/** Simple wildcard pattern matching for test delivery. */
function matchPattern(pattern: string, address: string): boolean {
  if (pattern === address) return true
  // Handle trailing /* wildcard
  if (pattern.endsWith('/*')) {
    const prefix = pattern.slice(0, -2)
    if (!address.startsWith(prefix + '/')) return false
    // Only match one level deep
    return !address.slice(prefix.length + 1).includes('/')
  }
  return false
}

function createConfig(store: MemoryKeyStore, overrides?: Partial<E2EConfig>): E2EConfig {
  return {
    identityId: 'alice',
    basePath: '/test/room/1',
    store,
    ...overrides,
  }
}

describe('E2ESession', () => {
  let store: MemoryKeyStore
  let client: ReturnType<typeof createMockClient>

  beforeEach(() => {
    store = new MemoryKeyStore()
    client = createMockClient()
  })

  it('starts without a group key', async () => {
    const session = new E2ESession(client, createConfig(store))
    await session.start()
    expect(session.encrypted).toBe(false)
  })

  it('enableEncryption creates a group key', async () => {
    const session = new E2ESession(client, createConfig(store))
    await session.start()
    await session.enableEncryption()
    expect(session.encrypted).toBe(true)
  })

  it('persists group key to store', async () => {
    const session = new E2ESession(client, createConfig(store))
    await session.start()
    await session.enableEncryption()

    const stored = await store.loadGroupKey('/test/room/1')
    expect(stored).not.toBeNull()
    expect(stored!.key).toBeDefined()
    expect(stored!.storedAt).toBeGreaterThan(0)
  })

  it('loads persisted group key on start', async () => {
    // First session creates the key
    const session1 = new E2ESession(client, createConfig(store))
    await session1.start()
    await session1.enableEncryption()

    // Encrypt a message
    const envelope = await session1.encrypt('hello')
    expect(envelope).not.toBeNull()
    session1.destroy()

    // Second session loads it
    const session2 = new E2ESession(client, createConfig(store))
    await session2.start()
    expect(session2.encrypted).toBe(true)

    // Should decrypt with the loaded key
    const decrypted = await session2.decrypt(envelope!)
    expect(decrypted).toBe('hello')
  })

  it('encrypt returns null without a group key', async () => {
    const session = new E2ESession(client, createConfig(store))
    await session.start()
    const result = await session.encrypt('test')
    expect(result).toBeNull()
  })

  it('encrypt/decrypt round-trips', async () => {
    const session = new E2ESession(client, createConfig(store))
    await session.start()
    await session.enableEncryption()

    const envelope = await session.encrypt('Hello, world!')
    expect(envelope).not.toBeNull()
    expect(envelope!._e2e).toBe(1)
    expect(envelope!.v).toBe(1)
    expect(typeof envelope!.ct).toBe('string')
    expect(typeof envelope!.iv).toBe('string')

    const decrypted = await session.decrypt(envelope!)
    expect(decrypted).toBe('Hello, world!')
  })

  it('decrypt returns null for tampered ciphertext', async () => {
    const session = new E2ESession(client, createConfig(store))
    await session.start()
    await session.enableEncryption()

    const envelope = await session.encrypt('secret')
    expect(envelope).not.toBeNull()

    // Tamper with ciphertext
    const tampered: E2EEnvelope = { ...envelope!, ct: 'AAAA' + envelope!.ct.slice(4) }
    const result = await session.decrypt(tampered)
    expect(result).toBeNull()
  })

  it('publishes ECDH public key on enableEncryption', async () => {
    const session = new E2ESession(client, createConfig(store))
    await session.start()
    await session.enableEncryption()

    const pubkeySets = client._sets.filter((s) =>
      s.address.includes('/_e2e/pubkey/')
    )
    expect(pubkeySets.length).toBeGreaterThan(0)
    const set = pubkeySets[0]
    expect(set.address).toBe('/test/room/1/_e2e/pubkey/alice')
    expect((set.value as Record<string, unknown>).publicKey).toBeDefined()
  })

  it('publishes ECDH public key on requestGroupKey', async () => {
    const session = new E2ESession(client, createConfig(store))
    await session.start()
    await session.requestGroupKey()

    const pubkeySets = client._sets.filter((s) =>
      s.address.includes('/_e2e/pubkey/')
    )
    expect(pubkeySets.length).toBeGreaterThan(0)
  })

  it('skips requestGroupKey if already has key', async () => {
    const session = new E2ESession(client, createConfig(store))
    await session.start()
    await session.enableEncryption()
    const countBefore = client._sets.length
    await session.requestGroupKey()
    expect(client._sets.length).toBe(countBefore) // no new sets
  })

  it('distributes group key to peer on pubkey subscription', async () => {
    // Alice has the group key
    const session = new E2ESession(client, createConfig(store))
    await session.start()
    await session.enableEncryption()

    // Bob publishes their ECDH public key (simulated via mock)
    const { generateECDHKeyPair, exportKey } = await import('../src/primitives')
    const bobKp = await generateECDHKeyPair()
    const bobPubJwk = await exportKey(bobKp.publicKey)

    // Deliver Bob's pubkey to Alice's subscription
    client._deliver('/test/room/1/_e2e/pubkey/bob', {
      publicKey: bobPubJwk,
      timestamp: Date.now(),
    })

    // Wait for async key exchange to complete
    await new Promise((r) => setTimeout(r, 50))

    // Alice should have emitted an encrypted key exchange to Bob
    const keyexEmits = client._emits.filter((e) =>
      e.address === '/test/room/1/_e2e/keyex/bob'
    )
    expect(keyexEmits.length).toBe(1)
    const payload = keyexEmits[0].payload as Record<string, unknown>
    expect(payload.fromId).toBe('alice')
    expect(payload.encryptedKey).toBeDefined()
    expect(payload.iv).toBeDefined()
    expect(payload.senderPublicKey).toBeDefined()
  })

  it('rotateKey generates new key and redistributes', async () => {
    const session = new E2ESession(client, createConfig(store))
    await session.start()
    await session.enableEncryption()

    const envelope1 = await session.encrypt('before rotation')

    await session.rotateKey()
    expect(session.encrypted).toBe(true)

    // New key should produce different ciphertext
    const envelope2 = await session.encrypt('after rotation')
    expect(envelope2!.ct).not.toBe(envelope1!.ct)

    // Old ciphertext should not decrypt with new key
    const result = await session.decrypt(envelope1!)
    expect(result).toBeNull()
  })

  it('removePeer clears cached public key', async () => {
    const session = new E2ESession(client, createConfig(store))
    await session.start()
    await session.enableEncryption()

    // Simulate a peer publishing their key
    const { generateECDHKeyPair, exportKey } = await import('../src/primitives')
    const bobKp = await generateECDHKeyPair()
    client._deliver('/test/room/1/_e2e/pubkey/bob', {
      publicKey: await exportKey(bobKp.publicKey),
      timestamp: Date.now(),
    })
    await new Promise((r) => setTimeout(r, 50))

    // Remove Bob
    session.removePeer('bob')

    // Rotate — Bob should not receive the new key
    const emitsBefore = client._emits.length
    await session.rotateKey()
    await new Promise((r) => setTimeout(r, 50))

    const keyexToBob = client._emits
      .slice(emitsBefore)
      .filter((e) => e.address === '/test/room/1/_e2e/keyex/bob')
    expect(keyexToBob.length).toBe(0)
  })

  it('TOFU calls onKeyChange when fingerprint changes', async () => {
    const onKeyChange = vi.fn().mockReturnValue(true)
    const session = new E2ESession(
      client,
      createConfig(store, { onKeyChange })
    )
    await session.start()
    await session.enableEncryption()

    const { generateECDHKeyPair, exportKey } = await import('../src/primitives')

    // First key from Bob — should be trusted (no callback)
    const bobKp1 = await generateECDHKeyPair()
    client._deliver('/test/room/1/_e2e/pubkey/bob', {
      publicKey: await exportKey(bobKp1.publicKey),
      timestamp: Date.now(),
    })
    await new Promise((r) => setTimeout(r, 50))
    expect(onKeyChange).not.toHaveBeenCalled()

    // Second different key from Bob — should trigger warning, accepted
    const bobKp2 = await generateECDHKeyPair()
    client._deliver('/test/room/1/_e2e/pubkey/bob', {
      publicKey: await exportKey(bobKp2.publicKey),
      timestamp: Date.now(),
    })
    await new Promise((r) => setTimeout(r, 50))
    expect(onKeyChange).toHaveBeenCalledWith(
      'bob',
      expect.any(String),
      expect.any(String)
    )
    // Old and new fingerprints should be different
    const [, oldFp, newFp] = onKeyChange.mock.calls[0]
    expect(oldFp).not.toBe(newFp)
  })

  it('TOFU rejects key change without callback', async () => {
    const session = new E2ESession(
      client,
      createConfig(store) // no onKeyChange
    )
    await session.start()
    await session.enableEncryption()

    const { generateECDHKeyPair, exportKey } = await import('../src/primitives')

    // First key from Bob — trusted
    const bobKp1 = await generateECDHKeyPair()
    client._deliver('/test/room/1/_e2e/pubkey/bob', {
      publicKey: await exportKey(bobKp1.publicKey),
      timestamp: Date.now(),
    })
    await new Promise((r) => setTimeout(r, 50))

    // Second key from Bob — should be rejected (no callback)
    const bobKp2 = await generateECDHKeyPair()
    client._deliver('/test/room/1/_e2e/pubkey/bob', {
      publicKey: await exportKey(bobKp2.publicKey),
      timestamp: Date.now(),
    })
    // The rejection happens inside the subscribe callback; since there's
    // no way to observe the rejection directly from the mock, we verify
    // that no key exchange was emitted for bob's second key
    await new Promise((r) => setTimeout(r, 50))
    const keyexEmits = client._emits.filter((e) =>
      e.address === '/test/room/1/_e2e/keyex/bob'
    )
    // Only the first key exchange should have happened
    expect(keyexEmits.length).toBe(1)
  })

  it('destroy cleans up subscriptions', async () => {
    const session = new E2ESession(client, createConfig(store))
    await session.start()
    await session.enableEncryption()
    session.destroy()

    expect(session.encrypted).toBe(false)
    await expect(session.enableEncryption()).rejects.toThrow('destroyed')
  })

  it('encrypt/decrypt throw after destroy', async () => {
    const session = new E2ESession(client, createConfig(store))
    await session.start()
    await session.enableEncryption()
    const envelope = await session.encrypt('test')
    session.destroy()

    await expect(session.encrypt('test')).rejects.toThrow('destroyed')
    await expect(session.decrypt(envelope!)).rejects.toThrow('destroyed')
  })

  it('rejects key exchange with empty fromId', async () => {
    const session = new E2ESession(client, createConfig(store))
    await session.start()

    // Simulate receiving a keyex message with empty fromId
    const { generateECDHKeyPair, exportKey } = await import('../src/primitives')
    const bobKp = await generateECDHKeyPair()

    client._deliver('/test/room/1/_e2e/keyex/alice', {
      fromId: '',
      encryptedKey: 'AAAA',
      iv: 'BBBB',
      senderPublicKey: await exportKey(bobKp.publicKey),
    })
    await new Promise((r) => setTimeout(r, 50))

    // Should not have accepted a group key (no key from empty sender)
    expect(session.encrypted).toBe(false)
  })

  it('rejects key exchange with missing fromId', async () => {
    const session = new E2ESession(client, createConfig(store))
    await session.start()

    client._deliver('/test/room/1/_e2e/keyex/alice', {
      encryptedKey: 'AAAA',
      iv: 'BBBB',
      senderPublicKey: {},
    })
    await new Promise((r) => setTimeout(r, 50))

    expect(session.encrypted).toBe(false)
  })

  it('decrypt returns null for unknown envelope version', async () => {
    const session = new E2ESession(client, createConfig(store))
    await session.start()
    await session.enableEncryption()

    const envelope = await session.encrypt('test')
    expect(envelope).not.toBeNull()

    // Tamper version
    const futureVersionEnvelope: E2EEnvelope = { ...envelope!, v: 2 }
    const result = await session.decrypt(futureVersionEnvelope)
    expect(result).toBeNull()
  })

  it('throws on operations after destroy', async () => {
    const session = new E2ESession(client, createConfig(store))
    session.destroy()
    await expect(session.start()).rejects.toThrow('destroyed')
    await expect(session.enableEncryption()).rejects.toThrow('destroyed')
    await expect(session.requestGroupKey()).rejects.toThrow('destroyed')
    await expect(session.rotateKey()).rejects.toThrow('destroyed')
  })
})
