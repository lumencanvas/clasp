/**
 * E2E encryption primitives using Web Crypto API.
 * AES-256-GCM for symmetric encryption, ECDH P-256 for key exchange,
 * ECDSA P-256 for signing, HKDF-SHA256 for key derivation.
 *
 * Ported from apps/chat/src/lib/crypto.js with TypeScript types.
 * Returns Uint8Array instead of base64 strings — caller decides encoding.
 */

// TS 5.9+ workaround: Uint8Array<ArrayBufferLike> is not directly assignable
// to BufferSource. This is safe — Web Crypto accepts Uint8Array at runtime.
const buf = (data: Uint8Array): BufferSource => data as unknown as BufferSource

// --- AES-256-GCM ---

/** Generate a random AES-256-GCM key for group/room encryption. */
export async function generateGroupKey(): Promise<CryptoKey> {
  return crypto.subtle.generateKey(
    { name: 'AES-GCM', length: 256 },
    true,
    ['encrypt', 'decrypt']
  )
}

/** Encrypt plaintext with AES-256-GCM. Returns ciphertext and random 12-byte IV. */
export async function encrypt(
  key: CryptoKey,
  plaintext: Uint8Array
): Promise<{ ciphertext: Uint8Array; iv: Uint8Array }> {
  const iv = crypto.getRandomValues(new Uint8Array(12))
  const encrypted = await crypto.subtle.encrypt(
    { name: 'AES-GCM', iv },
    key,
    buf(plaintext)
  )
  return { ciphertext: new Uint8Array(encrypted), iv }
}

/** Decrypt ciphertext with AES-256-GCM. */
export async function decrypt(
  key: CryptoKey,
  ciphertext: Uint8Array,
  iv: Uint8Array
): Promise<Uint8Array> {
  const decrypted = await crypto.subtle.decrypt(
    { name: 'AES-GCM', iv: buf(iv) } as AesGcmParams,
    key,
    buf(ciphertext)
  )
  return new Uint8Array(decrypted)
}

// --- ECDH P-256 ---

/** Generate an ECDH P-256 key pair. Private key is non-extractable. */
export async function generateECDHKeyPair(): Promise<{
  publicKey: CryptoKey
  privateKey: CryptoKey
}> {
  const keyPair = await crypto.subtle.generateKey(
    { name: 'ECDH', namedCurve: 'P-256' },
    false,
    ['deriveKey', 'deriveBits']
  )
  return { publicKey: keyPair.publicKey, privateKey: keyPair.privateKey }
}

/**
 * Derive a shared AES-256-GCM key from ECDH key exchange via HKDF-SHA256.
 * Raw ECDH output has biased bits; HKDF provides proper key derivation
 * with domain separation.
 */
export async function deriveSharedKey(
  privateKey: CryptoKey,
  peerPublicKey: CryptoKey,
  info: string = 'clasp-e2e-keyex-v1'
): Promise<CryptoKey> {
  const rawBits = await crypto.subtle.deriveBits(
    { name: 'ECDH', public: peerPublicKey },
    privateKey,
    256
  )
  const hkdfKey = await crypto.subtle.importKey(
    'raw',
    rawBits,
    'HKDF',
    false,
    ['deriveKey']
  )
  const infoBytes = new TextEncoder().encode(info)
  return crypto.subtle.deriveKey(
    {
      name: 'HKDF',
      hash: 'SHA-256',
      salt: buf(new Uint8Array(32)),
      info: buf(infoBytes),
    } as HkdfParams,
    hkdfKey,
    { name: 'AES-GCM', length: 256 },
    false,
    ['encrypt', 'decrypt']
  )
}

// --- ECDSA P-256 ---

/** Generate an ECDSA P-256 key pair for signing. Private key is non-extractable. */
export async function generateSigningKeyPair(): Promise<{
  publicKey: CryptoKey
  privateKey: CryptoKey
}> {
  const keyPair = await crypto.subtle.generateKey(
    { name: 'ECDSA', namedCurve: 'P-256' },
    false,
    ['sign', 'verify']
  )
  return { publicKey: keyPair.publicKey, privateKey: keyPair.privateKey }
}

/** Sign data with an ECDSA P-256 private key. */
export async function sign(
  privateKey: CryptoKey,
  data: Uint8Array
): Promise<Uint8Array> {
  const signature = await crypto.subtle.sign(
    { name: 'ECDSA', hash: 'SHA-256' },
    privateKey,
    buf(data)
  )
  return new Uint8Array(signature)
}

/** Verify an ECDSA P-256 signature. */
export async function verify(
  publicKey: CryptoKey,
  data: Uint8Array,
  signature: Uint8Array
): Promise<boolean> {
  return crypto.subtle.verify(
    { name: 'ECDSA', hash: 'SHA-256' },
    publicKey,
    buf(signature),
    buf(data)
  )
}

// --- Key import/export ---

/** Export a CryptoKey to JWK format. */
export async function exportKey(key: CryptoKey): Promise<JsonWebKey> {
  return crypto.subtle.exportKey('jwk', key)
}

/** Import an AES-256-GCM group key from JWK. */
export async function importGroupKey(jwk: JsonWebKey): Promise<CryptoKey> {
  return crypto.subtle.importKey(
    'jwk',
    jwk,
    { name: 'AES-GCM', length: 256 },
    true,
    ['encrypt', 'decrypt']
  )
}

/** Import an ECDH P-256 public key from JWK. */
export async function importECDHPublicKey(jwk: JsonWebKey): Promise<CryptoKey> {
  return crypto.subtle.importKey(
    'jwk',
    jwk,
    { name: 'ECDH', namedCurve: 'P-256' },
    true,
    []
  )
}

/** Import an ECDH P-256 private key from JWK. Non-extractable. */
export async function importECDHPrivateKey(jwk: JsonWebKey): Promise<CryptoKey> {
  return crypto.subtle.importKey(
    'jwk',
    jwk,
    { name: 'ECDH', namedCurve: 'P-256' },
    false,
    ['deriveKey', 'deriveBits']
  )
}

/** Import an ECDSA P-256 public key from JWK. */
export async function importSigningPublicKey(jwk: JsonWebKey): Promise<CryptoKey> {
  return crypto.subtle.importKey(
    'jwk',
    jwk,
    { name: 'ECDSA', namedCurve: 'P-256' },
    true,
    ['verify']
  )
}

// --- Fingerprinting ---

/**
 * Compute a SHA-256 fingerprint of a JWK public key.
 * Normalizes to identity-relevant fields only ({crv, kty, x, y} for EC keys)
 * to ensure cross-platform interop between JS and Rust implementations.
 * Returns hex string in groups of 4 for display (e.g. "a1b2 c3d4 e5f6 ...").
 */
export async function fingerprint(jwk: JsonWebKey): Promise<string> {
  // Normalize to identity-relevant fields for deterministic fingerprinting
  const normalized: Record<string, unknown> = jwk.kty === 'EC'
    ? { crv: jwk.crv, kty: jwk.kty, x: jwk.x, y: jwk.y }
    : { ...(jwk as Record<string, unknown>) }
  const canonical = JSON.stringify(normalized, Object.keys(normalized).sort())
  const data = new TextEncoder().encode(canonical)
  const hash = await crypto.subtle.digest('SHA-256', buf(data))
  const bytes = new Uint8Array(hash)
  const hex = Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('')
  return (hex.match(/.{1,4}/g) ?? []).join(' ')
}

// --- Constant-time comparison ---

/** Constant-time string comparison to prevent timing side-channels. */
export function constantTimeEqual(a: string, b: string): boolean {
  if (a.length !== b.length) return false
  let result = 0
  for (let i = 0; i < a.length; i++) {
    result |= a.charCodeAt(i) ^ b.charCodeAt(i)
  }
  return result === 0
}

// --- Encoding helpers ---

/** Convert Uint8Array to base64 string. */
export function toBase64(bytes: Uint8Array): string {
  let binary = ''
  for (let i = 0; i < bytes.byteLength; i++) {
    binary += String.fromCharCode(bytes[i])
  }
  return btoa(binary)
}

/** Convert base64 string to Uint8Array. */
export function fromBase64(base64: string): Uint8Array {
  const binary = atob(base64)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i)
  }
  return bytes
}
