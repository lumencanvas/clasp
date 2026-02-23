import type { E2EConfig, E2EEnvelope, KeyStore, TofuRecord } from './types'
import {
  generateGroupKey,
  generateECDHKeyPair,
  deriveSharedKey,
  encrypt,
  decrypt,
  exportKey,
  importGroupKey,
  importECDHPublicKey,
  fingerprint,
  constantTimeEqual,
  toBase64,
  fromBase64,
} from './primitives'

/**
 * Minimal interface for the CLASP client operations E2ESession needs.
 * Matches the Clasp class from @clasp-to/core without importing it directly.
 */
export interface ClaspLike {
  set(address: string, value: unknown): void
  emit(address: string, payload?: unknown): void
  subscribe(pattern: string, callback: (data: unknown, address: string) => void): () => void
  get connected(): boolean
}

/**
 * E2E encryption session for a single group/room/channel.
 * Manages ECDH key exchange, group key distribution, TOFU verification,
 * and encrypt/decrypt operations over CLASP paths.
 *
 * Generalized from apps/chat/src/composables/useCrypto.js — framework-agnostic.
 */
export class E2ESession {
  private client: ClaspLike
  private config: E2EConfig
  private store: KeyStore

  private groupKey: CryptoKey | null = null
  private ecdhKeyPairPromise: Promise<{ publicKey: CryptoKey; privateKey: CryptoKey }> | null = null
  private peerPublicKeys = new Map<string, JsonWebKey>()
  private unsubscribes: (() => void)[] = []
  private started = false
  private destroyed = false

  constructor(client: ClaspLike, config: E2EConfig) {
    this.client = client
    this.config = config
    this.store = config.store
  }

  /** Whether this session has an active group key. */
  get encrypted(): boolean {
    return this.groupKey !== null
  }

  /** The base path used for E2E subpaths. */
  get basePath(): string {
    return this.config.basePath
  }

  /**
   * Start the session: subscribe to key exchange paths and attempt
   * to load a persisted group key from the store.
   */
  async start(): Promise<void> {
    if (this.destroyed) throw new Error('E2ESession has been destroyed')
    if (this.started) return
    this.started = true

    // Try loading persisted key
    const sessionId = this.sessionId()
    const stored = await this.store.loadGroupKey(sessionId)
    if (stored) {
      try {
        this.groupKey = await importGroupKey(stored.key)
      } catch {
        // Stored key is corrupted, delete it
        await this.store.deleteGroupKey(sessionId)
      }
    }

    // Subscribe to key exchange events
    this.subscribeKeyExchange()
  }

  /**
   * Enable encryption: generate a new group key and start distributing it.
   * Publishes the ECDH public key so peers can request the key.
   */
  async enableEncryption(): Promise<void> {
    if (this.destroyed) throw new Error('E2ESession has been destroyed')
    if (!this.started) await this.start()

    this.groupKey = await generateGroupKey()

    // Persist
    const exported = await exportKey(this.groupKey)
    await this.store.saveGroupKey(this.sessionId(), {
      key: exported,
      storedAt: Date.now(),
    })

    // Publish our ECDH public key to let peers know we're available
    await this.publishPublicKey()
  }

  /**
   * Request the group key from peers by publishing our ECDH public key.
   * Called when joining an encrypted group without a key.
   */
  async requestGroupKey(): Promise<void> {
    if (this.destroyed) throw new Error('E2ESession has been destroyed')
    if (this.groupKey) return
    await this.publishPublicKey()
  }

  /**
   * Encrypt a string value into an E2EEnvelope.
   * Returns null if no group key is available.
   */
  async encrypt(value: string): Promise<E2EEnvelope | null> {
    if (this.destroyed) throw new Error('E2ESession has been destroyed')
    if (!this.groupKey) return null
    const plaintext = new TextEncoder().encode(value)
    const { ciphertext, iv } = await encrypt(this.groupKey, plaintext)
    return {
      _e2e: 1,
      ct: toBase64(ciphertext),
      iv: toBase64(iv),
      v: 1,
    }
  }

  /**
   * Decrypt an E2EEnvelope back to a string.
   * Returns null if decryption fails (wrong key, tampering, no key).
   */
  async decrypt(envelope: E2EEnvelope): Promise<string | null> {
    if (this.destroyed) throw new Error('E2ESession has been destroyed')
    if (envelope.v !== 1) return null // Unknown envelope version
    const key = this.groupKey ?? await this.tryLoadKey()
    if (!key) return null
    try {
      const ciphertext = fromBase64(envelope.ct)
      const iv = fromBase64(envelope.iv)
      const plaintext = await decrypt(key, ciphertext, iv)
      return new TextDecoder().decode(plaintext)
    } catch {
      return null
    }
  }

  /**
   * Rotate the group key: generate a new key and proactively distribute
   * to all cached peers. Use after removing a member.
   */
  async rotateKey(): Promise<void> {
    if (this.destroyed) throw new Error('E2ESession has been destroyed')
    if (!this.groupKey) return

    this.groupKey = await generateGroupKey()

    // Persist
    const exported = await exportKey(this.groupKey)
    await this.store.saveGroupKey(this.sessionId(), {
      key: exported,
      storedAt: Date.now(),
    })

    // Re-publish our public key
    await this.publishPublicKey()

    // Proactively distribute to cached peers
    await this.distributeKeyToPeers()
  }

  /** Remove a peer's cached public key (e.g., after banning). */
  removePeer(peerId: string): void {
    this.peerPublicKeys.delete(peerId)
  }

  /** Clean up subscriptions. */
  destroy(): void {
    this.destroyed = true
    for (const unsub of this.unsubscribes) {
      unsub()
    }
    this.unsubscribes = []
    this.groupKey = null
    this.ecdhKeyPairPromise = null
    this.peerPublicKeys.clear()
  }

  // --- Private ---

  private sessionId(): string {
    return this.config.basePath
  }

  private getECDHKeyPair(): Promise<{ publicKey: CryptoKey; privateKey: CryptoKey }> {
    if (!this.ecdhKeyPairPromise) {
      this.ecdhKeyPairPromise = generateECDHKeyPair()
    }
    return this.ecdhKeyPairPromise
  }

  private async publishPublicKey(): Promise<void> {
    if (!this.client.connected) return
    const kp = await this.getECDHKeyPair()
    const pubJwk = await exportKey(kp.publicKey)
    this.client.set(
      `${this.config.basePath}/_e2e/pubkey/${this.config.identityId}`,
      { publicKey: pubJwk, timestamp: Date.now() }
    )
  }

  private subscribeKeyExchange(): void {
    const { basePath, identityId } = this.config

    // Watch for peer ECDH public keys
    const unsubPubkey = this.client.subscribe(
      `${basePath}/_e2e/pubkey/*`,
      async (data: unknown, address: string) => {
        try {
          if (!data || !this.client.connected || this.destroyed) return
          const peerId = address.split('/').pop()!
          if (peerId === identityId) return

          const msg = data as { publicKey?: JsonWebKey; timestamp?: number }
          if (!msg.publicKey) return

          // TOFU verification — may reject if key changed
          await this.verifyPeerKey(peerId, msg.publicKey)
          if (this.destroyed) return

          // Cache peer's public key for future rotation
          this.peerPublicKeys.set(peerId, msg.publicKey)

          // Only distribute if we have the group key
          if (!this.groupKey) return

          // Password gate check
          if (this.config.passwordHash) {
            const proof = await this.waitForPasswordProof(peerId)
            if (this.destroyed) return
            if (!proof || !constantTimeEqual(proof, this.config.passwordHash)) return
          }

          // Derive shared secret and send encrypted group key
          const peerPubKey = await importECDHPublicKey(msg.publicKey)
          const kp = await this.getECDHKeyPair()
          const sharedKey = await deriveSharedKey(kp.privateKey, peerPubKey)
          if (this.destroyed) return

          const groupKeyJwk = JSON.stringify(await exportKey(this.groupKey!))
          const plaintext = new TextEncoder().encode(groupKeyJwk)
          const { ciphertext, iv } = await encrypt(sharedKey, plaintext)
          // Zeroize plaintext key material
          plaintext.fill(0)

          this.client.emit(`${basePath}/_e2e/keyex/${peerId}`, {
            fromId: identityId,
            encryptedKey: toBase64(ciphertext),
            iv: toBase64(iv),
            senderPublicKey: await exportKey(kp.publicKey),
          })
        } catch {
          // TOFU rejection or key exchange failure — non-fatal
        }
      }
    )
    this.unsubscribes.push(unsubPubkey)

    // Watch for encrypted group keys sent to us
    const unsubKeyex = this.client.subscribe(
      `${basePath}/_e2e/keyex/${identityId}`,
      async (data: unknown) => {
        if (!data || this.destroyed) return
        // Only accept a group key if we don't already have one —
        // prevents rogue peers from replacing the active key.
        if (this.groupKey) return
        const msg = data as {
          fromId?: string
          encryptedKey?: string
          iv?: string
          senderPublicKey?: JsonWebKey
        }
        if (!msg.encryptedKey || !msg.iv || !msg.senderPublicKey) return

        try {
          // Reject empty sender ID — prevents TOFU bypass
          if (!msg.fromId) return
          await this.verifyPeerKey(msg.fromId, msg.senderPublicKey)
          if (this.destroyed) return

          // Derive shared secret
          const senderPubKey = await importECDHPublicKey(msg.senderPublicKey)
          const kp = await this.getECDHKeyPair()
          const sharedKey = await deriveSharedKey(kp.privateKey, senderPubKey)
          if (this.destroyed) return

          // Decrypt the group key
          const ciphertext = fromBase64(msg.encryptedKey)
          const iv = fromBase64(msg.iv)
          const decrypted = await decrypt(sharedKey, ciphertext, iv)
          const groupKeyJwk = JSON.parse(new TextDecoder().decode(decrypted))
          // Zeroize decrypted key material
          decrypted.fill(0)
          if (this.destroyed) return
          this.groupKey = await importGroupKey(groupKeyJwk)

          // Persist
          const exported = await exportKey(this.groupKey)
          await this.store.saveGroupKey(this.sessionId(), {
            key: exported,
            storedAt: Date.now(),
          })
        } catch {
          // Failed to receive group key — non-fatal
        }
      }
    )
    this.unsubscribes.push(unsubKeyex)
  }

  private async verifyPeerKey(peerId: string, publicKeyJwk: JsonWebKey): Promise<void> {
    const fp = await fingerprint(publicKeyJwk)
    const recordId = `${this.config.basePath}:${peerId}`
    const stored = await this.store.loadTofuRecord(recordId)

    if (!stored) {
      // First time — trust on first use, always store
      await this.store.saveTofuRecord(recordId, {
        fingerprint: fp,
        firstSeen: Date.now(),
      })
      return
    }

    if (!constantTimeEqual(stored.fingerprint, fp)) {
      // Key changed — check if caller accepts
      let accepted = false
      if (this.config.onKeyChange) {
        const result = this.config.onKeyChange(peerId, stored.fingerprint, fp)
        accepted = (result instanceof Promise ? await result : result) === true
      }
      if (!accepted) {
        throw new Error(`TOFU key change rejected for peer ${peerId}`)
      }
      // Update stored record to new fingerprint, preserving original firstSeen
      await this.store.saveTofuRecord(recordId, {
        fingerprint: fp,
        firstSeen: stored.firstSeen,
      })
    }
  }

  private waitForPasswordProof(peerId: string): Promise<string | null> {
    const { basePath } = this.config
    return new Promise((resolve) => {
      let settled = false
      const unsub = this.client.subscribe(
        `${basePath}/_e2e/proof/${peerId}`,
        (data: unknown) => {
          if (settled) return
          settled = true
          clearTimeout(timerId)
          unsub()
          const msg = data as { hash?: string } | null
          resolve(msg?.hash ?? null)
        }
      )
      const timerId = setTimeout(() => {
        if (settled) return
        settled = true
        unsub()
        resolve(null)
      }, 2000)
    })
  }

  private async distributeKeyToPeers(): Promise<void> {
    if (!this.groupKey || !this.client.connected) return

    const kp = await this.getECDHKeyPair()
    const groupKeyJwk = JSON.stringify(await exportKey(this.groupKey))
    const plaintext = new TextEncoder().encode(groupKeyJwk)
    const senderPubJwk = await exportKey(kp.publicKey)

    for (const [peerId, peerPubKeyJwk] of this.peerPublicKeys) {
      if (this.destroyed) break
      if (peerId === this.config.identityId) continue
      try {
        const peerPubKey = await importECDHPublicKey(peerPubKeyJwk)
        const sharedKey = await deriveSharedKey(kp.privateKey, peerPubKey)
        const { ciphertext, iv } = await encrypt(sharedKey, plaintext)
        this.client.emit(`${this.config.basePath}/_e2e/keyex/${peerId}`, {
          fromId: this.config.identityId,
          encryptedKey: toBase64(ciphertext),
          iv: toBase64(iv),
          senderPublicKey: senderPubJwk,
        })
      } catch {
        // Non-fatal: failed to distribute to one peer
      }
    }
    // Zeroize plaintext key material
    plaintext.fill(0)
  }

  private async tryLoadKey(): Promise<CryptoKey | null> {
    const stored = await this.store.loadGroupKey(this.sessionId())
    if (!stored) return null
    try {
      this.groupKey = await importGroupKey(stored.key)
      return this.groupKey
    } catch {
      return null
    }
  }
}
