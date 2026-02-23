// Primitives (Layer 1)
export {
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
  importECDHPrivateKey,
  importSigningPublicKey,
  fingerprint,
  constantTimeEqual,
  toBase64,
  fromBase64,
} from './primitives'

// Types
export type {
  E2EEnvelope,
  E2EConfig,
  CryptoClientConfig,
  KeyStore,
  KeyData,
  TofuRecord,
} from './types'

// Storage
export { MemoryKeyStore, IndexedDBKeyStore } from './storage'

// Protocol (Layer 2)
export { E2ESession } from './protocol'
export type { ClaspLike } from './protocol'

// Client wrapper
export { CryptoClient } from './client'
