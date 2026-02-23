import { describe, it, expect } from 'vitest'
import {
  generateGroupKey,
  encrypt,
  decrypt,
  generateECDHKeyPair,
  deriveSharedKey,
  generateSigningKeyPair,
  sign,
  verify,
  exportKey,
  importGroupKey,
  importECDHPublicKey,
  importSigningPublicKey,
  fingerprint,
  toBase64,
  fromBase64,
} from '../src/primitives'

describe('primitives', () => {
  describe('AES-256-GCM', () => {
    it('generateGroupKey returns a CryptoKey', async () => {
      const key = await generateGroupKey()
      expect(key).toBeInstanceOf(CryptoKey)
      expect(key.algorithm.name).toBe('AES-GCM')
      expect((key.algorithm as AesKeyAlgorithm).length).toBe(256)
      expect(key.extractable).toBe(true)
      expect(key.usages).toContain('encrypt')
      expect(key.usages).toContain('decrypt')
    })

    it('round-trips plaintext correctly', async () => {
      const key = await generateGroupKey()
      const plaintext = new TextEncoder().encode('Hello, world!')
      const { ciphertext, iv } = await encrypt(key, plaintext)
      const decrypted = await decrypt(key, ciphertext, iv)
      expect(new TextDecoder().decode(decrypted)).toBe('Hello, world!')
    })

    it('produces different ciphertext for same plaintext (random IV)', async () => {
      const key = await generateGroupKey()
      const plaintext = new TextEncoder().encode('Same message')
      const enc1 = await encrypt(key, plaintext)
      const enc2 = await encrypt(key, plaintext)
      expect(toBase64(enc1.ciphertext)).not.toBe(toBase64(enc2.ciphertext))
      expect(toBase64(enc1.iv)).not.toBe(toBase64(enc2.iv))
    })

    it('fails to decrypt with wrong key', async () => {
      const key1 = await generateGroupKey()
      const key2 = await generateGroupKey()
      const plaintext = new TextEncoder().encode('secret')
      const { ciphertext, iv } = await encrypt(key1, plaintext)
      await expect(decrypt(key2, ciphertext, iv)).rejects.toThrow()
    })

    it('handles unicode text', async () => {
      const key = await generateGroupKey()
      const text = 'Unicode: \u00e9\u00e8\u00ea \u4f60\u597d \ud83d\ude00'
      const plaintext = new TextEncoder().encode(text)
      const { ciphertext, iv } = await encrypt(key, plaintext)
      const decrypted = await decrypt(key, ciphertext, iv)
      expect(new TextDecoder().decode(decrypted)).toBe(text)
    })

    it('handles empty input', async () => {
      const key = await generateGroupKey()
      const plaintext = new Uint8Array(0)
      const { ciphertext, iv } = await encrypt(key, plaintext)
      const decrypted = await decrypt(key, ciphertext, iv)
      expect(decrypted.byteLength).toBe(0)
    })

    it('returns Uint8Array, not base64 strings', async () => {
      const key = await generateGroupKey()
      const { ciphertext, iv } = await encrypt(key, new Uint8Array([1, 2, 3]))
      expect(ciphertext).toBeInstanceOf(Uint8Array)
      expect(iv).toBeInstanceOf(Uint8Array)
      expect(iv.byteLength).toBe(12) // AES-GCM IV is 12 bytes
    })
  })

  describe('ECDH key exchange', () => {
    it('generates key pairs with correct algorithm', async () => {
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

    it('deriveSharedKey produces a working AES key', async () => {
      const kpA = await generateECDHKeyPair()
      const kpB = await generateECDHKeyPair()
      const shared = await deriveSharedKey(kpA.privateKey, kpB.publicKey)
      expect(shared).toBeInstanceOf(CryptoKey)
      expect(shared.algorithm.name).toBe('AES-GCM')
    })

    it('both sides derive the same shared secret', async () => {
      const kpA = await generateECDHKeyPair()
      const kpB = await generateECDHKeyPair()
      const sharedAB = await deriveSharedKey(kpA.privateKey, kpB.publicKey)
      const sharedBA = await deriveSharedKey(kpB.privateKey, kpA.publicKey)

      const plaintext = new TextEncoder().encode('test secret')
      const { ciphertext, iv } = await encrypt(sharedAB, plaintext)
      const decrypted = await decrypt(sharedBA, ciphertext, iv)
      expect(new TextDecoder().decode(decrypted)).toBe('test secret')
    })

    it('uses custom info parameter for domain separation', async () => {
      const kpA = await generateECDHKeyPair()
      const kpB = await generateECDHKeyPair()
      const key1 = await deriveSharedKey(kpA.privateKey, kpB.publicKey, 'domain-1')
      const key2 = await deriveSharedKey(kpA.privateKey, kpB.publicKey, 'domain-2')

      // Different info strings should produce different keys
      const plaintext = new TextEncoder().encode('test')
      const { ciphertext, iv } = await encrypt(key1, plaintext)
      await expect(decrypt(key2, ciphertext, iv)).rejects.toThrow()
    })
  })

  describe('ECDSA signing', () => {
    it('generates signing key pairs', async () => {
      const kp = await generateSigningKeyPair()
      expect(kp.publicKey).toBeInstanceOf(CryptoKey)
      expect(kp.privateKey).toBeInstanceOf(CryptoKey)
      expect(kp.publicKey.algorithm.name).toBe('ECDSA')
      expect(kp.privateKey.extractable).toBe(false)
    })

    it('sign and verify round-trips', async () => {
      const kp = await generateSigningKeyPair()
      const data = new TextEncoder().encode('message to sign')
      const sig = await sign(kp.privateKey, data)
      expect(sig).toBeInstanceOf(Uint8Array)

      const valid = await verify(kp.publicKey, data, sig)
      expect(valid).toBe(true)
    })

    it('fails verification with wrong key', async () => {
      const kp1 = await generateSigningKeyPair()
      const kp2 = await generateSigningKeyPair()
      const data = new TextEncoder().encode('message')
      const sig = await sign(kp1.privateKey, data)

      const valid = await verify(kp2.publicKey, data, sig)
      expect(valid).toBe(false)
    })

    it('fails verification with tampered data', async () => {
      const kp = await generateSigningKeyPair()
      const data = new TextEncoder().encode('original')
      const sig = await sign(kp.privateKey, data)

      const tampered = new TextEncoder().encode('tampered')
      const valid = await verify(kp.publicKey, tampered, sig)
      expect(valid).toBe(false)
    })
  })

  describe('key import/export', () => {
    it('round-trips AES group key', async () => {
      const key = await generateGroupKey()
      const jwk = await exportKey(key)
      const imported = await importGroupKey(jwk)
      expect(imported).toBeInstanceOf(CryptoKey)
      expect(imported.algorithm.name).toBe('AES-GCM')

      // Verify functional equivalence
      const plaintext = new TextEncoder().encode('test')
      const { ciphertext, iv } = await encrypt(key, plaintext)
      const decrypted = await decrypt(imported, ciphertext, iv)
      expect(new TextDecoder().decode(decrypted)).toBe('test')
    })

    it('round-trips ECDH public key', async () => {
      const kp = await generateECDHKeyPair()
      const jwk = await exportKey(kp.publicKey)
      const imported = await importECDHPublicKey(jwk)
      expect(imported).toBeInstanceOf(CryptoKey)
      expect(imported.algorithm.name).toBe('ECDH')
    })

    it('round-trips ECDSA signing public key', async () => {
      const kp = await generateSigningKeyPair()
      const jwk = await exportKey(kp.publicKey)
      const imported = await importSigningPublicKey(jwk)
      expect(imported).toBeInstanceOf(CryptoKey)
      expect(imported.algorithm.name).toBe('ECDSA')

      // Verify it works for verification
      const data = new TextEncoder().encode('test')
      const sig = await sign(kp.privateKey, data)
      const valid = await verify(imported, data, sig)
      expect(valid).toBe(true)
    })
  })

  describe('fingerprint', () => {
    it('produces consistent output for same key', async () => {
      const kp = await generateECDHKeyPair()
      const jwk = await exportKey(kp.publicKey)
      const fp1 = await fingerprint(jwk)
      const fp2 = await fingerprint(jwk)
      expect(fp1).toBe(fp2)
    })

    it('produces different output for different keys', async () => {
      const kp1 = await generateECDHKeyPair()
      const kp2 = await generateECDHKeyPair()
      const fp1 = await fingerprint(await exportKey(kp1.publicKey))
      const fp2 = await fingerprint(await exportKey(kp2.publicKey))
      expect(fp1).not.toBe(fp2)
    })

    it('returns hex groups of 4', async () => {
      const kp = await generateECDHKeyPair()
      const fp = await fingerprint(await exportKey(kp.publicKey))
      // SHA-256 = 64 hex chars = 16 groups of 4
      const groups = fp.split(' ')
      expect(groups.length).toBe(16)
      for (const g of groups) {
        expect(g).toMatch(/^[0-9a-f]{4}$/)
      }
    })
  })

  describe('base64 helpers', () => {
    it('round-trips Uint8Array', () => {
      const original = new Uint8Array([0, 1, 127, 128, 255])
      const encoded = toBase64(original)
      const decoded = fromBase64(encoded)
      expect(decoded).toEqual(original)
    })

    it('produces valid base64 strings', () => {
      const data = crypto.getRandomValues(new Uint8Array(32))
      const encoded = toBase64(data)
      expect(encoded).toMatch(/^[A-Za-z0-9+/=]+$/)
    })

    it('handles empty arrays', () => {
      const encoded = toBase64(new Uint8Array(0))
      const decoded = fromBase64(encoded)
      expect(decoded.byteLength).toBe(0)
    })
  })
})
