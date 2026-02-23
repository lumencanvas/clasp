import type { CryptoClientConfig, E2EConfig, E2EEnvelope } from './types'
import { E2ESession, type ClaspLike } from './protocol'

/** Check if a value is an E2E encrypted envelope. */
function isE2EEnvelope(value: unknown): value is E2EEnvelope {
  if (!value || typeof value !== 'object') return false
  const obj = value as Record<string, unknown>
  return obj._e2e === 1
    && typeof obj.ct === 'string'
    && typeof obj.iv === 'string'
    && typeof obj.v === 'number'
}

/**
 * CryptoClient wraps a Clasp instance to provide transparent E2E encryption.
 *
 * Usage:
 *   const crypto = new CryptoClient(clasp, { identityId, store })
 *   const session = crypto.session('/myapp/room/general')
 *   await session.start()
 *   await session.enableEncryption()
 *   crypto.emit('/myapp/room/general/messages', { text: 'hello' })
 *   crypto.subscribe('/myapp/room/general/messages', (data) => { ... })
 *
 * The `inner` property provides direct access to the wrapped Clasp client
 * for operations that should not be encrypted.
 */
export class CryptoClient {
  /** Direct access to the underlying Clasp client for unencrypted operations. */
  readonly inner: ClaspLike

  private config: CryptoClientConfig
  private sessions = new Map<string, E2ESession>()

  constructor(client: ClaspLike, config: CryptoClientConfig) {
    this.inner = client
    this.config = config
  }

  /**
   * Get or create an E2ESession for a base path.
   * The session manages key exchange and encryption for one group/room/channel.
   */
  session(basePath: string, options?: Partial<E2EConfig>): E2ESession {
    let s = this.sessions.get(basePath)
    if (!s) {
      s = new E2ESession(this.inner, {
        identityId: this.config.identityId,
        basePath,
        store: this.config.store,
        onKeyChange: this.config.onKeyChange,
        ...options,
      })
      this.sessions.set(basePath, s)
    }
    return s
  }

  /**
   * Set a value, encrypting it if a session exists for the address's base path.
   * Falls through to unencrypted set if no matching session or no group key.
   */
  async set(address: string, value: unknown): Promise<void> {
    const s = this.findSession(address)
    if (s?.encrypted && value !== undefined) {
      const text = typeof value === 'string' ? value : JSON.stringify(value)
      const envelope = await s.encrypt(text)
      if (envelope) {
        this.inner.set(address, envelope)
        return
      }
    }
    this.inner.set(address, value)
  }

  /**
   * Emit an event, encrypting the payload if a session exists.
   * Object payloads are JSON-stringified before encryption.
   */
  async emit(address: string, payload?: unknown): Promise<void> {
    const s = this.findSession(address)
    if (s?.encrypted && payload !== undefined) {
      const text = typeof payload === 'string' ? payload : JSON.stringify(payload)
      const envelope = await s.encrypt(text)
      if (envelope) {
        this.inner.emit(address, envelope)
        return
      }
    }
    this.inner.emit(address, payload)
  }

  /**
   * Subscribe to a pattern with automatic decryption of E2E envelopes.
   * If the received value is an E2EEnvelope, it is decrypted before
   * calling the callback. Non-envelope values pass through unchanged.
   */
  subscribe(
    pattern: string,
    callback: (data: unknown, address: string) => void
  ): () => void {
    return this.inner.subscribe(pattern, async (data: unknown, address: string) => {
      try {
        if (isE2EEnvelope(data)) {
          const s = this.findSession(address)
          if (s) {
            const decrypted = await s.decrypt(data)
            if (decrypted !== null) {
              // Try parsing as JSON, fall back to string
              try {
                callback(JSON.parse(decrypted), address)
              } catch {
                callback(decrypted, address)
              }
              return
            }
          }
          // Couldn't decrypt — pass envelope as-is
          callback(data, address)
        } else {
          callback(data, address)
        }
      } catch {
        // Decryption failed — pass data as-is
        callback(data, address)
      }
    })
  }

  /** Close all sessions and the underlying client. */
  close(): void {
    for (const s of this.sessions.values()) {
      s.destroy()
    }
    this.sessions.clear()
  }

  /**
   * Find the session whose basePath is a prefix of the given address.
   * Uses longest-prefix match if multiple sessions could match.
   */
  private findSession(address: string): E2ESession | null {
    let best: E2ESession | null = null
    let bestLen = 0
    for (const [basePath, session] of this.sessions) {
      if ((address === basePath || address.startsWith(basePath + '/')) && basePath.length > bestLen) {
        best = session
        bestLen = basePath.length
      }
    }
    return best
  }
}
