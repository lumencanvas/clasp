import { describe, it, expect, vi } from 'vitest'
import { Room } from '../src/room'

function mockCryptoClient() {
  return {
    set: vi.fn().mockResolvedValue(undefined),
    emit: vi.fn().mockResolvedValue(undefined),
    subscribe: vi.fn().mockReturnValue(() => {}),
    close: vi.fn(),
  }
}

function mockSession(overrides: Record<string, unknown> = {}) {
  return {
    encrypted: true,
    basePath: '/chat/private',
    lastRotation: Date.now(),
    rotationCount: 0,
    rotateKey: vi.fn().mockResolvedValue(undefined),
    removePeer: vi.fn(),
    destroy: vi.fn(),
    ...overrides,
  }
}

describe('Room', () => {
  describe('Data operations', () => {
    it('set() encrypts through crypto client', async () => {
      const crypto = mockCryptoClient()
      const session = mockSession()
      const room = new Room('/chat/private', crypto as any, session as any)

      await room.set('/chat/private/messages/1', { text: 'hello' })
      expect(crypto.set).toHaveBeenCalledWith('/chat/private/messages/1', { text: 'hello' })
    })

    it('set() validates address is under basePath', async () => {
      const crypto = mockCryptoClient()
      const session = mockSession()
      const room = new Room('/chat/private', crypto as any, session as any)

      await expect(room.set('/other/path', 42)).rejects.toThrow(
        'Address "/other/path" is not under room basePath "/chat/private"'
      )
    })

    it('set() allows exact basePath as address', async () => {
      const crypto = mockCryptoClient()
      const session = mockSession()
      const room = new Room('/chat/private', crypto as any, session as any)

      await room.set('/chat/private', { status: 'active' })
      expect(crypto.set).toHaveBeenCalled()
    })

    it('set() rejects address that is a prefix-substring but not path-prefix', async () => {
      const crypto = mockCryptoClient()
      const session = mockSession()
      const room = new Room('/chat/private', crypto as any, session as any)

      await expect(room.set('/chat/privately', 42)).rejects.toThrow('is not under room basePath')
    })

    it('emit() encrypts through crypto client', async () => {
      const crypto = mockCryptoClient()
      const session = mockSession()
      const room = new Room('/chat/private', crypto as any, session as any)

      await room.emit('/chat/private/typing', { user: 'alice' })
      expect(crypto.emit).toHaveBeenCalledWith('/chat/private/typing', { user: 'alice' })
    })

    it('emit() validates address under basePath', async () => {
      const crypto = mockCryptoClient()
      const session = mockSession()
      const room = new Room('/chat/private', crypto as any, session as any)

      await expect(room.emit('/other/path', {})).rejects.toThrow('is not under room basePath')
    })

    it('on() subscribes through crypto client', () => {
      const crypto = mockCryptoClient()
      const session = mockSession()
      const room = new Room('/chat/private', crypto as any, session as any)

      const cb = vi.fn()
      room.on('/chat/private/messages/**', cb)
      expect(crypto.subscribe).toHaveBeenCalledWith('/chat/private/messages/**', cb)
    })

    it('on() unsubscribe function works', () => {
      const unsubFn = vi.fn()
      const crypto = mockCryptoClient()
      crypto.subscribe = vi.fn().mockReturnValue(unsubFn)
      const session = mockSession()
      const room = new Room('/chat/private', crypto as any, session as any)

      const unsub = room.on('/chat/private/**', vi.fn())
      unsub()
      expect(unsubFn).toHaveBeenCalled()
    })
  })

  describe('Encryption state', () => {
    it('encrypted returns true when session has key', () => {
      const crypto = mockCryptoClient()
      const session = mockSession({ encrypted: true })
      const room = new Room('/chat/private', crypto as any, session as any)

      expect(room.encrypted).toBe(true)
    })

    it('encrypted returns false when session has no key', () => {
      const crypto = mockCryptoClient()
      const session = mockSession({ encrypted: false })
      const room = new Room('/chat/private', crypto as any, session as any)

      expect(room.encrypted).toBe(false)
    })

    it('lastRotation returns session value', () => {
      const ts = 1700000000000
      const crypto = mockCryptoClient()
      const session = mockSession({ lastRotation: ts })
      const room = new Room('/chat/private', crypto as any, session as any)

      expect(room.lastRotation).toBe(ts)
    })

    it('rotationCount returns session value', () => {
      const crypto = mockCryptoClient()
      const session = mockSession({ rotationCount: 5 })
      const room = new Room('/chat/private', crypto as any, session as any)

      expect(room.rotationCount).toBe(5)
    })

    it('rotateKey() delegates to session', async () => {
      const crypto = mockCryptoClient()
      const session = mockSession()
      const room = new Room('/chat/private', crypto as any, session as any)

      await room.rotateKey()
      expect(session.rotateKey).toHaveBeenCalled()
    })

    it('removePeer() delegates with correct peerId', () => {
      const crypto = mockCryptoClient()
      const session = mockSession()
      const room = new Room('/chat/private', crypto as any, session as any)

      room.removePeer('peer-abc')
      expect(session.removePeer).toHaveBeenCalledWith('peer-abc')
    })
  })

  describe('Lifecycle', () => {
    it('destroy() calls session.destroy', () => {
      const crypto = mockCryptoClient()
      const session = mockSession()
      const room = new Room('/chat/private', crypto as any, session as any)

      room.destroy()
      expect(session.destroy).toHaveBeenCalled()
    })

    it('destroy() does not throw if called twice', () => {
      const crypto = mockCryptoClient()
      const session = mockSession()
      const room = new Room('/chat/private', crypto as any, session as any)

      room.destroy()
      expect(() => room.destroy()).not.toThrow()
    })

    it('basePath is readonly', () => {
      const crypto = mockCryptoClient()
      const session = mockSession()
      const room = new Room('/chat/private', crypto as any, session as any)

      expect(room.basePath).toBe('/chat/private')
    })
  })
})
