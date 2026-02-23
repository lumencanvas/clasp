import type { KeyStore, KeyData, TofuRecord } from './types'

/**
 * In-memory KeyStore for testing or non-persistent use cases.
 */
export class MemoryKeyStore implements KeyStore {
  private groupKeys = new Map<string, KeyData>()
  private tofuRecords = new Map<string, TofuRecord>()

  async saveGroupKey(sessionId: string, data: KeyData): Promise<void> {
    this.groupKeys.set(sessionId, { key: { ...data.key }, storedAt: data.storedAt })
  }

  async loadGroupKey(sessionId: string): Promise<KeyData | null> {
    const stored = this.groupKeys.get(sessionId)
    if (!stored) return null
    return { key: { ...stored.key }, storedAt: stored.storedAt }
  }

  async deleteGroupKey(sessionId: string): Promise<void> {
    this.groupKeys.delete(sessionId)
  }

  async saveTofuRecord(id: string, record: TofuRecord): Promise<void> {
    this.tofuRecords.set(id, { ...record })
  }

  async loadTofuRecord(id: string): Promise<TofuRecord | null> {
    const stored = this.tofuRecords.get(id)
    if (!stored) return null
    return { ...stored }
  }
}

/**
 * IndexedDB-backed KeyStore for browser environments.
 * Creates a dedicated database per app name to avoid collisions.
 */
export class IndexedDBKeyStore implements KeyStore {
  private dbName: string
  private dbPromise: Promise<IDBDatabase> | null = null

  constructor(appName: string) {
    this.dbName = `clasp-crypto-${appName}`
  }

  private openDB(): Promise<IDBDatabase> {
    if (this.dbPromise) return this.dbPromise

    this.dbPromise = new Promise((resolve, reject) => {
      const request = indexedDB.open(this.dbName, 1)

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result
        if (!db.objectStoreNames.contains('group-keys')) {
          db.createObjectStore('group-keys', { keyPath: 'sessionId' })
        }
        if (!db.objectStoreNames.contains('tofu-records')) {
          db.createObjectStore('tofu-records', { keyPath: 'id' })
        }
      }

      request.onsuccess = () => resolve(request.result)
      request.onerror = () => {
        this.dbPromise = null // Allow retry on next call
        reject(request.error)
      }
    })

    return this.dbPromise
  }

  async saveGroupKey(sessionId: string, data: KeyData): Promise<void> {
    const db = await this.openDB()
    return new Promise((resolve, reject) => {
      const tx = db.transaction('group-keys', 'readwrite')
      tx.objectStore('group-keys').put({ sessionId, ...data })
      tx.oncomplete = () => resolve()
      tx.onerror = () => reject(tx.error)
    })
  }

  async loadGroupKey(sessionId: string): Promise<KeyData | null> {
    const db = await this.openDB()
    return new Promise((resolve, reject) => {
      const tx = db.transaction('group-keys', 'readonly')
      const request = tx.objectStore('group-keys').get(sessionId)
      request.onsuccess = () => {
        const result = request.result
        if (!result) return resolve(null)
        resolve({ key: result.key, storedAt: result.storedAt })
      }
      request.onerror = () => reject(request.error)
    })
  }

  async deleteGroupKey(sessionId: string): Promise<void> {
    const db = await this.openDB()
    return new Promise((resolve, reject) => {
      const tx = db.transaction('group-keys', 'readwrite')
      tx.objectStore('group-keys').delete(sessionId)
      tx.oncomplete = () => resolve()
      tx.onerror = () => reject(tx.error)
    })
  }

  async saveTofuRecord(id: string, record: TofuRecord): Promise<void> {
    const db = await this.openDB()
    return new Promise((resolve, reject) => {
      const tx = db.transaction('tofu-records', 'readwrite')
      tx.objectStore('tofu-records').put({ id, ...record })
      tx.oncomplete = () => resolve()
      tx.onerror = () => reject(tx.error)
    })
  }

  async loadTofuRecord(id: string): Promise<TofuRecord | null> {
    const db = await this.openDB()
    return new Promise((resolve, reject) => {
      const tx = db.transaction('tofu-records', 'readonly')
      const request = tx.objectStore('tofu-records').get(id)
      request.onsuccess = () => {
        const result = request.result
        if (!result) return resolve(null)
        resolve({ fingerprint: result.fingerprint, firstSeen: result.firstSeen })
      }
      request.onerror = () => reject(request.error)
    })
  }
}
