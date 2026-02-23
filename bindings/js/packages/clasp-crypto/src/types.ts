/** E2E encrypted envelope that flows through CLASP as a normal map value. */
export interface E2EEnvelope {
  _e2e: 1
  /** Base64-encoded ciphertext */
  ct: string
  /** Base64-encoded IV (12 bytes for AES-GCM) */
  iv: string
  /** Envelope version */
  v: number
}

/** Configuration for an E2ESession. */
export interface E2EConfig {
  /** Unique identity ID for this participant (e.g. user UUID) */
  identityId: string
  /** Base path for this encrypted group/room/channel */
  basePath: string
  /** Persistent key storage */
  store: KeyStore
  /** Called when a peer's public key changes (TOFU violation).
   *  Must return true to accept the new key, or false to reject.
   *  If absent, key changes are rejected by default. */
  onKeyChange?: (peerId: string, oldFingerprint: string, newFingerprint: string) => boolean | Promise<boolean>
  /**
   * If set, gate key exchange on a password proof.
   * The value is the expected hash that peers must present.
   */
  passwordHash?: string
}

/** Configuration for the CryptoClient wrapper. */
export interface CryptoClientConfig {
  /** Unique identity ID for this participant */
  identityId: string
  /** Persistent key storage */
  store: KeyStore
  /** Called when a peer's public key changes (TOFU violation).
   *  Must return true to accept the new key, or false to reject. */
  onKeyChange?: (peerId: string, oldFingerprint: string, newFingerprint: string) => boolean | Promise<boolean>
}

/** Stored key material in JWK format with metadata. */
export interface KeyData {
  /** The group/room key in JWK format */
  key: JsonWebKey
  /** When this key was stored */
  storedAt: number
}

/** Stored TOFU fingerprint record. */
export interface TofuRecord {
  /** Hex fingerprint of the peer's ECDH public key */
  fingerprint: string
  /** When this key was first seen */
  firstSeen: number
}

/** Pluggable persistence interface for crypto keys and TOFU records. */
export interface KeyStore {
  /** Save a group key for a session/group. */
  saveGroupKey(sessionId: string, data: KeyData): Promise<void>
  /** Load a group key for a session/group. */
  loadGroupKey(sessionId: string): Promise<KeyData | null>
  /** Delete a group key for a session/group. */
  deleteGroupKey(sessionId: string): Promise<void>
  /** Save a TOFU fingerprint record. */
  saveTofuRecord(id: string, record: TofuRecord): Promise<void>
  /** Load a TOFU fingerprint record. */
  loadTofuRecord(id: string): Promise<TofuRecord | null>
}

