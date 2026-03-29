/**
 * Identity format utilities for CLASP.
 *
 * Note: Full key generation requires the crypto module.
 * These utilities format existing keys into CLASP identity formats.
 */

/** Format a 32-byte public key as a CLASP EntityId (clasp:<base58-first-16-bytes>) */
export function toEntityId(publicKey: Uint8Array): string {
  const bytes = publicKey.slice(0, 16)
  return `clasp:${base58Encode(bytes)}`
}

/** Format a 32-byte Ed25519 public key as a DID (did:key:z6Mk...) */
export function toDid(publicKey: Uint8Array): string {
  // multicodec 0xed for Ed25519
  const multicodec = new Uint8Array([0xed, 0x01, ...publicKey])
  return `did:key:z${base58Encode(multicodec)}`
}

/** base58btc encoding (Bitcoin alphabet) */
function base58Encode(bytes: Uint8Array): string {
  const ALPHABET = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz'
  let num = BigInt(0)
  for (const byte of bytes) {
    num = num * 256n + BigInt(byte)
  }
  let encoded = ''
  while (num > 0n) {
    encoded = ALPHABET[Number(num % 58n)] + encoded
    num = num / 58n
  }
  // Leading zeros
  for (const byte of bytes) {
    if (byte !== 0) break
    encoded = '1' + encoded
  }
  return encoded || '1'
}
