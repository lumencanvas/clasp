import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ref } from 'vue'

// Mock dependencies
const mockSet = vi.fn()
const mockEmit = vi.fn()
const mockSubscribe = vi.fn(() => vi.fn())
const mockConnected = ref(true)
const mockUserId = ref('user-1')

vi.mock('../useClasp.js', () => ({
  useClasp: () => ({
    connected: mockConnected,
    set: mockSet,
    emit: mockEmit,
    subscribe: mockSubscribe,
  }),
}))

vi.mock('../useIdentity.js', () => ({
  useIdentity: () => ({
    userId: mockUserId,
  }),
}))

// Mock storage — track persisted keys
const storedKeys = new Map()
vi.mock('../../lib/storage.js', () => ({
  saveCryptoKey: vi.fn((roomId, key) => {
    storedKeys.set(roomId, key)
    return Promise.resolve()
  }),
  loadCryptoKey: vi.fn((roomId) => {
    return Promise.resolve(storedKeys.get(roomId) ?? null)
  }),
}))

// Use REAL crypto.js — no mocking
import { useCrypto } from '../useCrypto.js'
import { saveCryptoKey, loadCryptoKey } from '../../lib/storage.js'
import {
  generateECDHKeyPair,
  exportKey,
  importKey,
  deriveSharedSecret,
  encryptMessage,
  decryptMessage,
} from '../../lib/crypto.js'

// Helper: reset module-level state in useCrypto between tests
// useCrypto uses module-level Maps/Sets, so we need to clear them
function resetCryptoState() {
  const { encryptedRooms } = useCrypto()
  encryptedRooms.value = new Set()
}

describe('useCrypto', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    storedKeys.clear()
    mockConnected.value = true
    mockUserId.value = 'user-1'
    resetCryptoState()
  })

  describe('enableEncryption', () => {
    it('generates AES key and adds roomId to encryptedRooms', async () => {
      const { enableEncryption, encryptedRooms } = useCrypto()
      expect(encryptedRooms.value.has('room-1')).toBe(false)

      const key = await enableEncryption('room-1')
      expect(key).toBeTruthy()
      expect(key.type).toBe('secret')
      expect(encryptedRooms.value.has('room-1')).toBe(true)
    })

    it('persists key via saveCryptoKey', async () => {
      const { enableEncryption } = useCrypto()
      await enableEncryption('room-persist')

      expect(saveCryptoKey).toHaveBeenCalledWith('room-persist', expect.any(Object))
      expect(storedKeys.has('room-persist')).toBe(true)
    })

    it('publishes ECDH public key via set()', async () => {
      const { enableEncryption } = useCrypto()
      await enableEncryption('room-pub')

      expect(mockSet).toHaveBeenCalledWith(
        expect.stringContaining('/chat/room/room-pub/crypto/pubkey/user-1'),
        expect.objectContaining({
          publicKey: expect.any(Object),
          timestamp: expect.any(Number),
        })
      )
    })
  })

  describe('loadRoomKey', () => {
    it('returns null for unknown room', async () => {
      const { loadRoomKey } = useCrypto()
      const key = await loadRoomKey('nonexistent')
      expect(key).toBeNull()
    })

    it('imports persisted key and adds to encryptedRooms', async () => {
      const { loadRoomKey, encryptedRooms } = useCrypto()

      // Manually persist a valid AES key JWK into storage (simulating a previous session)
      const { generateRoomKey, exportKey: exportAesKey } = await import('../../lib/crypto.js')
      const testKey = await generateRoomKey()
      const jwk = await exportAesKey(testKey)
      // Use a room ID never used with enableEncryption so roomKeys Map is empty for it
      storedKeys.set('room-load-fresh', jwk)

      expect(encryptedRooms.value.has('room-load-fresh')).toBe(false)

      const loaded = await loadRoomKey('room-load-fresh')
      expect(loaded).toBeTruthy()
      expect(loaded.type).toBe('secret')
      expect(encryptedRooms.value.has('room-load-fresh')).toBe(true)
    })
  })

  describe('encrypt / decrypt', () => {
    it('encrypt returns { ciphertext, iv } for encrypted room', async () => {
      const { enableEncryption, encrypt } = useCrypto()
      await enableEncryption('room-enc')

      const result = await encrypt('room-enc', 'hello world')
      expect(result).toBeTruthy()
      expect(result.ciphertext).toBeTruthy()
      expect(result.iv).toBeTruthy()
    })

    it('encrypt returns null for non-encrypted room', async () => {
      const { encrypt } = useCrypto()
      const result = await encrypt('no-key-room', 'hello')
      expect(result).toBeNull()
    })

    it('decrypt roundtrips with encrypt for same room', async () => {
      const { enableEncryption, encrypt, decrypt } = useCrypto()
      await enableEncryption('room-rt')

      const encrypted = await encrypt('room-rt', 'secret message')
      const plaintext = await decrypt('room-rt', encrypted.ciphertext, encrypted.iv)
      expect(plaintext).toBe('secret message')
    })

    it('decrypt returns null for room without key', async () => {
      const { decrypt } = useCrypto()
      const result = await decrypt('unknown-room', 'abc', 'def')
      expect(result).toBeNull()
    })
  })

  describe('isEncrypted', () => {
    it('returns false initially, true after enableEncryption', async () => {
      const { isEncrypted, enableEncryption } = useCrypto()
      expect(isEncrypted('room-check')).toBe(false)

      await enableEncryption('room-check')
      expect(isEncrypted('room-check')).toBe(true)
    })
  })

  describe('markPasswordProtected', () => {
    it('adds room to password-protected set', async () => {
      // We can't directly inspect passwordRooms (module-private),
      // but we can verify via subscribeKeyExchange behavior later.
      // For now, just ensure it doesn't throw.
      const { markPasswordProtected } = useCrypto()
      expect(() => markPasswordProtected('room-pw', 'expected-hash')).not.toThrow()
    })
  })

  describe('rotateRoomKey', () => {
    it('generates NEW key different from old', async () => {
      const { enableEncryption, encrypt, rotateRoomKey } = useCrypto()
      await enableEncryption('room-rotate')

      const encrypted1 = await encrypt('room-rotate', 'before rotation')
      await rotateRoomKey('room-rotate')

      // Encrypting again should use a different key, so decrypting with old params should differ
      const encrypted2 = await encrypt('room-rotate', 'before rotation')
      // Different ciphertexts (different keys + different IVs)
      expect(encrypted2.ciphertext).not.toBe(encrypted1.ciphertext)
    })

    it('persists new key', async () => {
      const { enableEncryption, rotateRoomKey } = useCrypto()
      await enableEncryption('room-rotate2')

      vi.clearAllMocks()
      await rotateRoomKey('room-rotate2')

      expect(saveCryptoKey).toHaveBeenCalledWith('room-rotate2', expect.any(Object))
    })

    it('re-publishes public key', async () => {
      const { enableEncryption, rotateRoomKey } = useCrypto()
      await enableEncryption('room-rotate3')

      vi.clearAllMocks()
      await rotateRoomKey('room-rotate3')

      expect(mockSet).toHaveBeenCalledWith(
        expect.stringContaining('/chat/room/room-rotate3/crypto/pubkey/user-1'),
        expect.objectContaining({ publicKey: expect.any(Object) })
      )
    })
  })

  describe('subscribeKeyExchange', () => {
    it('registers subscriptions for pubkey and keyex paths', () => {
      const { subscribeKeyExchange } = useCrypto()
      subscribeKeyExchange('room-sub')

      expect(mockSubscribe).toHaveBeenCalledWith(
        expect.stringContaining('/chat/room/room-sub/crypto/pubkey/*'),
        expect.any(Function)
      )
      expect(mockSubscribe).toHaveBeenCalledWith(
        expect.stringContaining('/chat/room/room-sub/crypto/keyex/user-1'),
        expect.any(Function)
      )
    })

    it('returns unsubscribe function that cleans up both subs', () => {
      const unsubA = vi.fn()
      const unsubB = vi.fn()
      mockSubscribe.mockReturnValueOnce(unsubA).mockReturnValueOnce(unsubB)

      const { subscribeKeyExchange } = useCrypto()
      const unsub = subscribeKeyExchange('room-unsub')

      unsub()
      expect(unsubA).toHaveBeenCalled()
      expect(unsubB).toHaveBeenCalled()
    })
  })

  describe('integration: full ECDH key exchange', () => {
    it('peer A sends encrypted room key to peer B who can then decrypt messages', async () => {
      const { enableEncryption, encrypt, subscribeKeyExchange } = useCrypto()

      // Peer A enables encryption and subscribes to key exchange
      await enableEncryption('room-kex')
      subscribeKeyExchange('room-kex')

      // Capture the pubkey subscription callback
      const pubkeySubCall = mockSubscribe.mock.calls.find(
        c => typeof c[0] === 'string' && c[0].includes('/crypto/pubkey/*')
      )
      expect(pubkeySubCall).toBeTruthy()
      const onPubkey = pubkeySubCall[1]

      // Capture the keyex subscription callback
      const keyexSubCall = mockSubscribe.mock.calls.find(
        c => typeof c[0] === 'string' && c[0].includes('/crypto/keyex/user-1')
      )
      expect(keyexSubCall).toBeTruthy()

      // Generate Peer B's ECDH key pair
      const peerBKeyPair = await generateECDHKeyPair()
      const peerBPubJwk = await exportKey(peerBKeyPair.publicKey)

      // Simulate Peer B publishing their public key
      await onPubkey(
        { publicKey: peerBPubJwk, timestamp: Date.now() },
        '/chat/room/room-kex/crypto/pubkey/user-B'
      )

      // Peer A should have emitted an encrypted room key to Peer B
      expect(mockEmit).toHaveBeenCalledWith(
        expect.stringContaining('/crypto/keyex/user-B'),
        expect.objectContaining({
          fromId: 'user-1',
          encryptedKey: expect.any(String),
          iv: expect.any(String),
          senderPublicKey: expect.any(Object),
        })
      )

      // Now simulate Peer B receiving the encrypted key and decrypting it
      const keyexPayload = mockEmit.mock.calls.find(
        c => typeof c[0] === 'string' && c[0].includes('/crypto/keyex/user-B')
      )[1]

      // Peer B derives the shared secret
      const senderPubKey = await importKey(keyexPayload.senderPublicKey, 'ecdh-public')
      const sharedKey = await deriveSharedSecret(peerBKeyPair.privateKey, senderPubKey)

      // Peer B decrypts the room key
      const roomKeyJwk = await decryptMessage(sharedKey, keyexPayload.encryptedKey, keyexPayload.iv)
      const roomKey = await importKey(JSON.parse(roomKeyJwk), 'aes')

      // Verify Peer B can decrypt a message from Peer A
      const encrypted = await encrypt('room-kex', 'hello from A')
      const plaintext = await decryptMessage(roomKey, encrypted.ciphertext, encrypted.iv)
      expect(plaintext).toBe('hello from A')
    })
  })

  describe('password proof gating', () => {
    it('does not send key when peer has no proof for password-protected room', async () => {
      const { enableEncryption, markPasswordProtected, subscribeKeyExchange } = useCrypto()

      await enableEncryption('room-pw-gate')
      markPasswordProtected('room-pw-gate', 'expected-room-hash')
      subscribeKeyExchange('room-pw-gate')

      // Capture the pubkey subscription callback
      const pubkeySubCall = mockSubscribe.mock.calls.find(
        c => typeof c[0] === 'string' && c[0].includes('room-pw-gate/crypto/pubkey/*')
      )
      const onPubkey = pubkeySubCall[1]

      // The password proof subscribe will be called inside onPubkey;
      // we need the next mockSubscribe call to simulate "no proof" (callback resolves null after timeout)
      const proofUnsub = vi.fn()
      mockSubscribe.mockReturnValueOnce(proofUnsub)

      const peerKeyPair = await generateECDHKeyPair()
      const peerPubJwk = await exportKey(peerKeyPair.publicKey)

      mockEmit.mockClear()

      // Trigger peer pubkey — the proof subscription will timeout with null
      await onPubkey(
        { publicKey: peerPubJwk, timestamp: Date.now() },
        '/chat/room/room-pw-gate/crypto/pubkey/peer-no-proof'
      )

      // Wait for the 2000ms timeout in the password proof check
      await new Promise(r => setTimeout(r, 2100))

      // Should NOT have emitted the key
      const keyexEmit = mockEmit.mock.calls.find(
        c => typeof c[0] === 'string' && c[0].includes('/crypto/keyex/peer-no-proof')
      )
      expect(keyexEmit).toBeUndefined()
    }, 5000)

    it('sends key when peer has valid proof for password-protected room', async () => {
      const { enableEncryption, markPasswordProtected, subscribeKeyExchange } = useCrypto()

      await enableEncryption('room-pw-ok')
      markPasswordProtected('room-pw-ok', 'valid-proof-hash')
      subscribeKeyExchange('room-pw-ok')

      // Capture the pubkey subscription callback
      const pubkeySubCall = mockSubscribe.mock.calls.find(
        c => typeof c[0] === 'string' && c[0].includes('room-pw-ok/crypto/pubkey/*')
      )
      const onPubkey = pubkeySubCall[1]

      // Mock the proof subscription to immediately call back with a valid proof
      mockSubscribe.mockImplementationOnce((path, cb) => {
        // Immediately provide proof
        setTimeout(() => cb({ hash: 'valid-proof-hash' }), 10)
        return vi.fn()
      })

      const peerKeyPair = await generateECDHKeyPair()
      const peerPubJwk = await exportKey(peerKeyPair.publicKey)

      mockEmit.mockClear()

      await onPubkey(
        { publicKey: peerPubJwk, timestamp: Date.now() },
        '/chat/room/room-pw-ok/crypto/pubkey/peer-with-proof'
      )

      // Wait for the proof callback to fire
      await new Promise(r => setTimeout(r, 100))

      // Should have emitted the encrypted key
      const keyexEmit = mockEmit.mock.calls.find(
        c => typeof c[0] === 'string' && c[0].includes('/crypto/keyex/peer-with-proof')
      )
      expect(keyexEmit).toBeTruthy()
      expect(keyexEmit[1]).toEqual(
        expect.objectContaining({
          fromId: 'user-1',
          encryptedKey: expect.any(String),
          iv: expect.any(String),
        })
      )
    })
  })
})
