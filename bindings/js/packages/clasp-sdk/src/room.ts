import type { CryptoClient, E2ESession } from '@clasp-to/crypto'
import type { Value } from '@clasp-to/core'

/**
 * An encrypted room. All set/emit/on calls are automatically encrypted
 * using the underlying E2ESession's group key.
 *
 * @example
 * ```typescript
 * const room = await c.room('/chat/private')
 * room.set('/chat/private/messages/1', { text: 'hello' })
 * room.on('/chat/private/messages/**', (msg) => console.log(msg))
 * ```
 */
export class Room {
  readonly basePath: string
  private crypto: CryptoClient
  private session: E2ESession

  constructor(basePath: string, crypto: CryptoClient, session: E2ESession) {
    this.basePath = basePath
    this.crypto = crypto
    this.session = session
  }

  /** Whether this room has an active encryption key. */
  get encrypted(): boolean {
    return this.session.encrypted
  }

  /** Timestamp of last key rotation, or null. */
  get lastRotation(): number | null {
    return this.session.lastRotation
  }

  /** Number of key rotations in this session. */
  get rotationCount(): number {
    return this.session.rotationCount
  }

  /**
   * Set a value in the room. Automatically encrypted.
   * Address must be under the room's basePath.
   */
  async set(address: string, value: Value): Promise<void> {
    this.validateAddress(address)
    await this.crypto.set(address, value)
  }

  /**
   * Emit an event in the room. Automatically encrypted.
   * Address must be under the room's basePath.
   */
  async emit(address: string, payload?: Value): Promise<void> {
    this.validateAddress(address)
    await this.crypto.emit(address, payload)
  }

  /**
   * Subscribe to addresses in the room. Automatically decrypted.
   * Pattern must be under the room's basePath.
   */
  on(pattern: string, callback: (data: unknown, address: string) => void): () => void {
    return this.crypto.subscribe(pattern, callback)
  }

  /** Manually rotate the group encryption key. */
  async rotateKey(): Promise<void> {
    await this.session.rotateKey()
  }

  /** Remove a peer from the room (clears their cached key). */
  removePeer(peerId: string): void {
    this.session.removePeer(peerId)
  }

  /** Clean up this room's session. */
  destroy(): void {
    this.session.destroy()
  }

  private validateAddress(address: string): void {
    if (address !== this.basePath && !address.startsWith(this.basePath + '/')) {
      throw new Error(
        `Address "${address}" is not under room basePath "${this.basePath}". ` +
        `Room operations must use addresses starting with "${this.basePath}/".`
      )
    }
  }
}
