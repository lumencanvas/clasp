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
    ['deriveKey', 'deriveBits']
  )
  // Public key still extractable for exchange; private key is non-extractable
  return {
    publicKey: keyPair.publicKey,
    privateKey: keyPair.privateKey,
  }
}

/**
 * Derive a shared AES key from ECDH key exchange via HKDF-SHA256.
 * Security (H3): raw ECDH output has biased bits; HKDF provides proper
 * key derivation with domain separation.
 */
export async function deriveSharedSecret(privateKey, peerPublicKey) {
  // Step 1: derive raw shared bits from ECDH
  const rawBits = await crypto.subtle.deriveBits(
    { name: 'ECDH', public: peerPublicKey },
    privateKey,
    256
  )

  // Step 2: import raw bits as HKDF key material
  const hkdfKey = await crypto.subtle.importKey(
    'raw', rawBits, 'HKDF', false, ['deriveKey']
  )

  // Step 3: derive AES-256-GCM key via HKDF-SHA256
  const info = new TextEncoder().encode('clasp-chat-keyex-v1')
  return crypto.subtle.deriveKey(
    { name: 'HKDF', hash: 'SHA-256', salt: new Uint8Array(32), info },
    hkdfKey,
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
      false,  // Security (M7): private keys should not be extractable
      ['deriveKey', 'deriveBits']
    )
  }
}

/**
 * Hash a password with PBKDF2 (SHA-256, 600k iterations per OWASP 2024).
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
      iterations: 600000,
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

/**
 * Generate an ECDSA P-256 key pair for message signing.
 * Private key is non-extractable for security.
 */
export async function generateSigningKeyPair() {
  const keyPair = await crypto.subtle.generateKey(
    { name: 'ECDSA', namedCurve: 'P-256' },
    false,
    ['sign', 'verify']
  )
  return {
    publicKey: keyPair.publicKey,
    privateKey: keyPair.privateKey,
  }
}

/**
 * Sign data with an ECDSA P-256 private key.
 * @param {CryptoKey} privateKey - ECDSA private key
 * @param {string} data - string data to sign
 * @returns {string} base64-encoded signature
 */
export async function signData(privateKey, data) {
  const encoded = new TextEncoder().encode(data)
  const signature = await crypto.subtle.sign(
    { name: 'ECDSA', hash: 'SHA-256' },
    privateKey,
    encoded
  )
  return arrayBufferToBase64(signature)
}

/**
 * Verify an ECDSA P-256 signature.
 * @param {CryptoKey} publicKey - ECDSA public key
 * @param {string} data - original string data
 * @param {string} signature - base64-encoded signature
 * @returns {boolean}
 */
export async function verifySignature(publicKey, data, signature) {
  const encoded = new TextEncoder().encode(data)
  const sigBytes = base64ToArrayBuffer(signature)
  return crypto.subtle.verify(
    { name: 'ECDSA', hash: 'SHA-256' },
    publicKey,
    sigBytes,
    encoded
  )
}

/**
 * Import an ECDSA public key from JWK format.
 */
export async function importSigningPublicKey(jwk) {
  return crypto.subtle.importKey(
    'jwk', jwk,
    { name: 'ECDSA', namedCurve: 'P-256' },
    true,
    ['verify']
  )
}

/**
 * Compute a SHA-256 fingerprint of a JWK public key.
 * Returns hex string in groups of 4 for display.
 */
export async function computeKeyFingerprint(jwk) {
  const canonical = JSON.stringify(jwk, Object.keys(jwk).sort())
  const data = new TextEncoder().encode(canonical)
  const hash = await crypto.subtle.digest('SHA-256', data)
  const bytes = new Uint8Array(hash)
  const hex = Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('')
  // Format as groups of 4 for readability: "a1b2 c3d4 e5f6 ..."
  return hex.match(/.{1,4}/g).join(' ')
}

// --- Helpers ---

export function arrayBufferToBase64(buffer) {
  const bytes = new Uint8Array(buffer)
  let binary = ''
  for (let i = 0; i < bytes.byteLength; i++) {
    binary += String.fromCharCode(bytes[i])
  }
  return btoa(binary)
}

export function base64ToArrayBuffer(base64) {
  const binary = atob(base64)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i)
  }
  return bytes
}
