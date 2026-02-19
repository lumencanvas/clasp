import { describe, it, expect, vi } from 'vitest'
import 'fake-indexeddb/auto'

// Each test uses unique room IDs to avoid cross-test interference.
// The DB is shared across tests (module-cached), which is fine since
// we use unique keys per test.

import {
  saveMessage,
  loadMessages,
  deleteRoomMessages,
  saveCryptoKey,
  loadCryptoKey,
  exportAll,
  importAll,
} from '../../lib/storage.js'

describe('storage', () => {
  describe('saveMessage + loadMessages', () => {
    it('roundtrip: save and load messages', async () => {
      const msg = {
        msgId: 'rt-msg-1',
        timestamp: 1000,
        from: 'Alice',
        text: 'hello',
      }

      await saveMessage('rt-room', msg)
      const loaded = await loadMessages('rt-room')

      expect(loaded).toHaveLength(1)
      expect(loaded[0].from).toBe('Alice')
      expect(loaded[0].text).toBe('hello')
    })

    it('returns messages ordered oldest-first', async () => {
      await saveMessage('order-room', { msgId: 'o-3', timestamp: 3000, text: 'third' })
      await saveMessage('order-room', { msgId: 'o-1', timestamp: 1000, text: 'first' })
      await saveMessage('order-room', { msgId: 'o-2', timestamp: 2000, text: 'second' })

      const loaded = await loadMessages('order-room')

      expect(loaded.map(m => m.text)).toEqual(['first', 'second', 'third'])
    })

    it('respects limit parameter', async () => {
      for (let i = 0; i < 10; i++) {
        await saveMessage('limit-room', {
          msgId: `lim-${i}`,
          timestamp: i * 1000,
          text: `message ${i}`,
        })
      }

      const loaded = await loadMessages('limit-room', 3)

      expect(loaded).toHaveLength(3)
      // Should return the NEWEST 3, ordered oldest-first
      expect(loaded.map(m => m.text)).toEqual(['message 7', 'message 8', 'message 9'])
    })
  })

  describe('deleteRoomMessages', () => {
    it('removes all messages for a room', async () => {
      await saveMessage('del-room-1', { msgId: 'dr-1', timestamp: 1000, text: 'a' })
      await saveMessage('del-room-1', { msgId: 'dr-2', timestamp: 2000, text: 'b' })
      await saveMessage('del-room-2', { msgId: 'dr-3', timestamp: 1000, text: 'c' })

      await deleteRoomMessages('del-room-1')

      // Wait for the cursor-based delete to complete
      await new Promise(r => setTimeout(r, 100))

      const room1 = await loadMessages('del-room-1')
      const room2 = await loadMessages('del-room-2')

      expect(room1).toHaveLength(0)
      expect(room2).toHaveLength(1)
    })
  })

  describe('saveCryptoKey + loadCryptoKey', () => {
    it('roundtrip: save and load crypto key', async () => {
      const keyData = { kty: 'oct', k: 'test-key-data', alg: 'A256GCM' }
      await saveCryptoKey('crypto-room-1', keyData)

      const loaded = await loadCryptoKey('crypto-room-1')

      expect(loaded).toEqual(keyData)
    })

    it('returns null for unknown room', async () => {
      const loaded = await loadCryptoKey('nonexistent-crypto-room')
      expect(loaded).toBeNull()
    })
  })

  describe('exportAll', () => {
    it('returns messages but NO crypto keys', async () => {
      await saveMessage('export-room', { msgId: 'exp-1', timestamp: 1000, text: 'hello' })
      await saveCryptoKey('export-room', { kty: 'oct', k: 'secret' })

      const exported = await exportAll()

      expect(exported.messages).toBeTruthy()
      expect(exported.messages.length).toBeGreaterThan(0)
      // Should NOT contain crypto keys
      expect(exported.cryptoKeys).toBeUndefined()
    })
  })

  describe('importAll', () => {
    it('restores messages and crypto keys (backwards compat)', async () => {
      const importData = {
        messages: [
          { roomId: 'import-room', msgId: 'imp-1', timestamp: 5000, data: { text: 'imported', msgId: 'imp-1', timestamp: 5000 } },
        ],
        cryptoKeys: [
          { roomId: 'import-room', key: { kty: 'oct', k: 'imported-key' } },
        ],
      }

      await importAll(importData)

      const msgs = await loadMessages('import-room')
      expect(msgs).toHaveLength(1)
      expect(msgs[0].text).toBe('imported')

      const key = await loadCryptoKey('import-room')
      expect(key).toEqual({ kty: 'oct', k: 'imported-key' })
    })
  })
})
