/**
 * E2E encryption utilities using Web Crypto API.
 * AES-256-GCM for message encryption, ECDH P-256 for key exchange.
 */

/**
 * Generate a random AES-256-GCM key for room encryption.
 */
export async function generateRoomKey() {
  return crypto.subtle.generateKey(
    { name: 'AES-GCM', length: 256 },
    true, // extractable for export
    ['encrypt', 'decrypt']
  )
}

/**
 * Encrypt plaintext with AES-256-GCM.
 * Returns { ciphertext, iv } as base64 strings.
 */
export async function encryptMessage(key, plaintext) {
  const encoder = new TextEncoder()
  const data = encoder.encode(plaintext)
  const iv = crypto.getRandomValues(new Uint8Array(12))

  const encrypted = await crypto.subtle.encrypt(
    { name: 'AES-GCM', iv },
    key,
    data
  )

  return {
    ciphertext: arrayBufferToBase64(encrypted),
    iv: arrayBufferToBase64(iv),
  }
}

/**
 * Decrypt ciphertext with AES-256-GCM.
 */
export async function decryptMessage(key, ciphertext, iv) {
  const data = base64ToArrayBuffer(ciphertext)
  const ivBytes = base64ToArrayBuffer(iv)

  const decrypted = await crypto.subtle.decrypt(
    { name: 'AES-GCM', iv: ivBytes },
    key,
    data
  )

  return new TextDecoder().decode(decrypted)
}

/**
 * Generate an ECDH key pair for key exchange.
 */
export async function generateECDHKeyPair() {
  const keyPair = await crypto.subtle.generateKey(
    { name: 'ECDH', namedCurve: 'P-256' },
    false,
    ['deriveKey']
  )
  // Public key still extractable for exchange; private key is non-extractable
  return {
    publicKey: keyPair.publicKey,
    privateKey: keyPair.privateKey,
  }
}

/**
 * Derive a shared AES key from ECDH key exchange.
 */
export async function deriveSharedSecret(privateKey, peerPublicKey) {
  return crypto.subtle.deriveKey(
    { name: 'ECDH', public: peerPublicKey },
    privateKey,
    { name: 'AES-GCM', length: 256 },
    true,
    ['encrypt', 'decrypt']
  )
}

/**
 * Export a CryptoKey to a JSON-safe format (JWK).
 */
export async function exportKey(key) {
  return crypto.subtle.exportKey('jwk', key)
}

/**
 * Import a key from JWK format.
 * @param {object} jwk - The JWK object
 * @param {'aes'|'ecdh-public'|'ecdh-private'} type
 */
export async function importKey(jwk, type = 'aes') {
  if (type === 'aes') {
    return crypto.subtle.importKey(
      'jwk', jwk,
      { name: 'AES-GCM', length: 256 },
      true,
      ['encrypt', 'decrypt']
    )
  }
  if (type === 'ecdh-public') {
    return crypto.subtle.importKey(
      'jwk', jwk,
      { name: 'ECDH', namedCurve: 'P-256' },
      true,
      []
    )
  }
  if (type === 'ecdh-private') {
    return crypto.subtle.importKey(
      'jwk', jwk,
      { name: 'ECDH', namedCurve: 'P-256' },
      true,
      ['deriveKey']
    )
  }
}

/**
 * Hash a password with PBKDF2 (SHA-256, 100k iterations).
 * Returns base64 hash string.
 */
export async function hashPassword(password, salt) {
  const encoder = new TextEncoder()
  const keyMaterial = await crypto.subtle.importKey(
    'raw',
    encoder.encode(password),
    'PBKDF2',
    false,
    ['deriveBits']
  )
  const bits = await crypto.subtle.deriveBits(
    {
      name: 'PBKDF2',
      salt: encoder.encode(salt),
      iterations: 100000,
      hash: 'SHA-256',
    },
    keyMaterial,
    256
  )
  return arrayBufferToBase64(bits)
}

/**
 * Generate a random salt string (base64-encoded 16 bytes).
 */
export function generateSalt() {
  const bytes = crypto.getRandomValues(new Uint8Array(16))
  return arrayBufferToBase64(bytes.buffer)
}

// --- Helpers ---

function arrayBufferToBase64(buffer) {
  const bytes = new Uint8Array(buffer)
  let binary = ''
  for (let i = 0; i < bytes.byteLength; i++) {
    binary += String.fromCharCode(bytes[i])
  }
  return btoa(binary)
}

function base64ToArrayBuffer(base64) {
  const binary = atob(base64)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i)
  }
  return bytes
}
