import { describe, it, expect } from 'vitest'
import {
  generateRoomKey,
  encryptMessage,
  decryptMessage,
  generateECDHKeyPair,
  deriveSharedSecret,
  exportKey,
  importKey,
  hashPassword,
  generateSalt,
} from '../crypto.js'

describe('crypto.js', () => {
  describe('generateRoomKey', () => {
    it('returns a CryptoKey with AES-GCM algorithm', async () => {
      const key = await generateRoomKey()
      expect(key).toBeInstanceOf(CryptoKey)
      expect(key.algorithm.name).toBe('AES-GCM')
      expect(key.algorithm.length).toBe(256)
    })

    it('generates extractable keys', async () => {
      const key = await generateRoomKey()
      expect(key.extractable).toBe(true)
    })

    it('supports encrypt and decrypt usages', async () => {
      const key = await generateRoomKey()
      expect(key.usages).toContain('encrypt')
      expect(key.usages).toContain('decrypt')
    })
  })

  describe('encryptMessage / decryptMessage', () => {
    it('round-trips plaintext correctly', async () => {
      const key = await generateRoomKey()
      const plaintext = 'Hello, world!'
      const { ciphertext, iv } = await encryptMessage(key, plaintext)
      const decrypted = await decryptMessage(key, ciphertext, iv)
      expect(decrypted).toBe(plaintext)
    })

    it('produces different ciphertext for same plaintext (random IV)', async () => {
      const key = await generateRoomKey()
      const plaintext = 'Same message'
      const enc1 = await encryptMessage(key, plaintext)
      const enc2 = await encryptMessage(key, plaintext)
      expect(enc1.ciphertext).not.toBe(enc2.ciphertext)
      expect(enc1.iv).not.toBe(enc2.iv)
    })

    it('returns base64 strings', async () => {
      const key = await generateRoomKey()
      const { ciphertext, iv } = await encryptMessage(key, 'test')
      // Base64 characters only
      expect(ciphertext).toMatch(/^[A-Za-z0-9+/=]+$/)
      expect(iv).toMatch(/^[A-Za-z0-9+/=]+$/)
    })

    it('fails to decrypt with wrong key', async () => {
      const key1 = await generateRoomKey()
      const key2 = await generateRoomKey()
      const { ciphertext, iv } = await encryptMessage(key1, 'secret')
      await expect(decryptMessage(key2, ciphertext, iv)).rejects.toThrow()
    })

    it('handles unicode text', async () => {
      const key = await generateRoomKey()
      const plaintext = 'Unicode: \u00e9\u00e8\u00ea \u4f60\u597d \ud83d\ude00'
      const { ciphertext, iv } = await encryptMessage(key, plaintext)
      const decrypted = await decryptMessage(key, ciphertext, iv)
      expect(decrypted).toBe(plaintext)
    })

    it('handles empty string', async () => {
      const key = await generateRoomKey()
      const { ciphertext, iv } = await encryptMessage(key, '')
      const decrypted = await decryptMessage(key, ciphertext, iv)
      expect(decrypted).toBe('')
    })
  })

  describe('ECDH key exchange', () => {
    it('generateECDHKeyPair returns public + private keys', async () => {
      const kp = await generateECDHKeyPair()
      expect(kp.publicKey).toBeInstanceOf(CryptoKey)
      expect(kp.privateKey).toBeInstanceOf(CryptoKey)
      expect(kp.publicKey.algorithm.name).toBe('ECDH')
      expect(kp.privateKey.algorithm.name).toBe('ECDH')
    })

    it('private key is non-extractable', async () => {
      const kp = await generateECDHKeyPair()
      expect(kp.privateKey.extractable).toBe(false)
    })

    it('deriveSharedSecret produces a working AES key', async () => {
      const kpA = await generateECDHKeyPair()
      const kpB = await generateECDHKeyPair()
      const shared = await deriveSharedSecret(kpA.privateKey, kpB.publicKey)
      expect(shared).toBeInstanceOf(CryptoKey)
      expect(shared.algorithm.name).toBe('AES-GCM')
    })

    it('both sides derive the same shared secret', async () => {
      const kpA = await generateECDHKeyPair()
      const kpB = await generateECDHKeyPair()
      const sharedAB = await deriveSharedSecret(kpA.privateKey, kpB.publicKey)
      const sharedBA = await deriveSharedSecret(kpB.privateKey, kpA.publicKey)

      // Encrypt with one, decrypt with the other
      const { ciphertext, iv } = await encryptMessage(sharedAB, 'test secret')
      const decrypted = await decryptMessage(sharedBA, ciphertext, iv)
      expect(decrypted).toBe('test secret')
    })
  })

  describe('exportKey / importKey', () => {
    it('round-trips AES key', async () => {
      const key = await generateRoomKey()
      const jwk = await exportKey(key)
      const imported = await importKey(jwk, 'aes')
      expect(imported).toBeInstanceOf(CryptoKey)
      expect(imported.algorithm.name).toBe('AES-GCM')

      // Verify it works for encryption
      const { ciphertext, iv } = await encryptMessage(key, 'test')
      const decrypted = await decryptMessage(imported, ciphertext, iv)
      expect(decrypted).toBe('test')
    })

    it('round-trips ECDH public key', async () => {
      const kp = await generateECDHKeyPair()
      const jwk = await exportKey(kp.publicKey)
      const imported = await importKey(jwk, 'ecdh-public')
      expect(imported).toBeInstanceOf(CryptoKey)
      expect(imported.algorithm.name).toBe('ECDH')
    })
  })

  describe('hashPassword', () => {
    it('produces consistent output for same input', async () => {
      const salt = 'test-salt-123'
      const hash1 = await hashPassword('mypassword', salt)
      const hash2 = await hashPassword('mypassword', salt)
      expect(hash1).toBe(hash2)
    })

    it('produces different output for different passwords', async () => {
      const salt = 'same-salt'
      const hash1 = await hashPassword('password1', salt)
      const hash2 = await hashPassword('password2', salt)
      expect(hash1).not.toBe(hash2)
    })

    it('produces different output for different salts', async () => {
      const hash1 = await hashPassword('same-password', 'salt1')
      const hash2 = await hashPassword('same-password', 'salt2')
      expect(hash1).not.toBe(hash2)
    })

    it('returns a base64 string', async () => {
      const hash = await hashPassword('test', 'salt')
      expect(hash).toMatch(/^[A-Za-z0-9+/=]+$/)
    })
  })

  describe('generateSalt', () => {
    it('returns a base64 string', () => {
      const salt = generateSalt()
      expect(salt).toMatch(/^[A-Za-z0-9+/=]+$/)
    })

    it('generates unique salts', () => {
      const salt1 = generateSalt()
      const salt2 = generateSalt()
      expect(salt1).not.toBe(salt2)
    })
  })
})
