import { describe, it, expect, beforeEach } from 'vitest'
import { CryptoClient } from '../src/client'
import { E2ESession, type ClaspLike } from '../src/protocol'
import { MemoryKeyStore } from '../src/storage'
import type { CryptoClientConfig, E2EEnvelope } from '../src/types'

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
  if (pattern.endsWith('/**')) {
    const prefix = pattern.slice(0, -3)
    return address === prefix || address.startsWith(prefix + '/')
  }
  return false
}

describe('CryptoClient', () => {
  let client: ReturnType<typeof createMockClient>
  let store: MemoryKeyStore
  let config: CryptoClientConfig

  beforeEach(() => {
    client = createMockClient()
    store = new MemoryKeyStore()
    config = { identityId: 'alice', store }
  })

  it('constructor stores config and wraps inner client', () => {
    const crypto = new CryptoClient(client, config)
    expect(crypto.inner).toBe(client)
  })

  it('session() creates E2ESession for a path', () => {
    const crypto = new CryptoClient(client, config)
    const session = crypto.session('/room/1')
    expect(session).toBeInstanceOf(E2ESession)
    expect(session.basePath).toBe('/room/1')
  })

  it('session() returns same session for same path', () => {
    const crypto = new CryptoClient(client, config)
    const s1 = crypto.session('/room/1')
    const s2 = crypto.session('/room/1')
    expect(s1).toBe(s2)
  })

  it('session() returns different sessions for different paths', () => {
    const crypto = new CryptoClient(client, config)
    const s1 = crypto.session('/room/1')
    const s2 = crypto.session('/room/2')
    expect(s1).not.toBe(s2)
  })

  it('set() encrypts when session active', async () => {
    const crypto = new CryptoClient(client, config)
    const session = crypto.session('/room/1')
    await session.start()
    await session.enableEncryption()

    const setsBefore = client._sets.length
    await crypto.set('/room/1/fader', 0.5)

    // Should have produced an encrypted envelope
    const newSets = client._sets.slice(setsBefore)
    expect(newSets.length).toBe(1)
    const value = newSets[0].value as Record<string, unknown>
    expect(value._e2e).toBe(1)
    expect(value.ct).toBeDefined()
    expect(value.iv).toBeDefined()
  })

  it('set() passes through when no session', async () => {
    const crypto = new CryptoClient(client, config)

    const setsBefore = client._sets.length
    await crypto.set('/other/path', 42)

    const newSets = client._sets.slice(setsBefore)
    expect(newSets.length).toBe(1)
    expect(newSets[0].value).toBe(42)
  })

  it('set() passes through when session has no key', async () => {
    const crypto = new CryptoClient(client, config)
    crypto.session('/room/1') // create session but don't enable encryption
    await crypto.session('/room/1').start()

    const setsBefore = client._sets.length
    await crypto.set('/room/1/fader', 0.5)

    const newSets = client._sets.slice(setsBefore)
    expect(newSets.length).toBe(1)
    expect(newSets[0].value).toBe(0.5)
  })

  it('emit() encrypts when session active', async () => {
    const crypto = new CryptoClient(client, config)
    const session = crypto.session('/room/1')
    await session.start()
    await session.enableEncryption()

    const emitsBefore = client._emits.length
    await crypto.emit('/room/1/trigger', { note: 60 })

    const newEmits = client._emits.slice(emitsBefore)
    // The keyex emits happen during start/enable, so filter for our address
    const triggerEmits = newEmits.filter(e => e.address === '/room/1/trigger')
    expect(triggerEmits.length).toBe(1)
    const payload = triggerEmits[0].payload as Record<string, unknown>
    expect(payload._e2e).toBe(1)
  })

  it('emit() passes through when no session', async () => {
    const crypto = new CryptoClient(client, config)

    await crypto.emit('/other/path', 'hello')

    const emits = client._emits.filter(e => e.address === '/other/path')
    expect(emits.length).toBe(1)
    expect(emits[0].payload).toBe('hello')
  })

  it('subscribe() auto-decrypts E2E envelopes', async () => {
    const crypto = new CryptoClient(client, config)
    const session = crypto.session('/room/1')
    await session.start()
    await session.enableEncryption()

    // Encrypt a value
    const envelope = await session.encrypt('secret message')
    expect(envelope).not.toBeNull()

    // Subscribe and deliver the encrypted envelope
    const received: unknown[] = []
    crypto.subscribe('/room/1/**', (data) => {
      received.push(data)
    })

    client._deliver('/room/1/messages', envelope)
    await new Promise(r => setTimeout(r, 50))

    expect(received.length).toBe(1)
    expect(received[0]).toBe('secret message')
  })

  it('subscribe() passes through non-envelope values', async () => {
    const crypto = new CryptoClient(client, config)

    const received: unknown[] = []
    crypto.subscribe('/room/1/**', (data) => {
      received.push(data)
    })

    client._deliver('/room/1/messages', { text: 'plain' })
    await new Promise(r => setTimeout(r, 50))

    expect(received.length).toBe(1)
    expect(received[0]).toEqual({ text: 'plain' })
  })

  it('findSession uses longest-prefix matching', async () => {
    const crypto = new CryptoClient(client, config)
    const general = crypto.session('/room')
    const specific = crypto.session('/room/vip')
    await general.start()
    await general.enableEncryption()
    await specific.start()
    await specific.enableEncryption()

    // /room/vip/fader should match /room/vip (more specific)
    const setsBefore = client._sets.length
    await crypto.set('/room/vip/fader', 0.8)

    // Verify it encrypted (meaning it found a session)
    const newSets = client._sets.slice(setsBefore)
    const faderSet = newSets.find(s => s.address === '/room/vip/fader')
    expect(faderSet).toBeDefined()
    const value = faderSet!.value as Record<string, unknown>
    expect(value._e2e).toBe(1)
  })

  it('close() destroys all sessions', async () => {
    const crypto = new CryptoClient(client, config)
    const s1 = crypto.session('/room/1')
    const s2 = crypto.session('/room/2')
    await s1.start()
    await s1.enableEncryption()
    await s2.start()
    await s2.enableEncryption()

    expect(s1.encrypted).toBe(true)
    expect(s2.encrypted).toBe(true)

    crypto.close()

    expect(s1.encrypted).toBe(false)
    expect(s2.encrypted).toBe(false)
  })
})
