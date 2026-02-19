/**
 * IndexedDB wrapper for CLASP Chat persistent storage.
 * DB: clasp-chat, version 2
 * Stores: messages, crypto-keys, tofu-keys, signing-keys
 */

const DB_NAME = 'clasp-chat'
const DB_VERSION = 2

let dbPromise = null

function openDB() {
  if (dbPromise) return dbPromise

  dbPromise = new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION)

    request.onupgradeneeded = (event) => {
      const db = event.target.result

      // Messages store
      if (!db.objectStoreNames.contains('messages')) {
        const msgStore = db.createObjectStore('messages', { keyPath: ['roomId', 'msgId'] })
        msgStore.createIndex('byRoom', ['roomId', 'timestamp'])
      }

      // Crypto keys store
      if (!db.objectStoreNames.contains('crypto-keys')) {
        db.createObjectStore('crypto-keys', { keyPath: 'roomId' })
      }

      // TOFU key fingerprints (Trust On First Use)
      if (!db.objectStoreNames.contains('tofu-keys')) {
        db.createObjectStore('tofu-keys', { keyPath: 'id' })
      }

      // ECDSA signing keys
      if (!db.objectStoreNames.contains('signing-keys')) {
        db.createObjectStore('signing-keys', { keyPath: 'userId' })
      }
    }

    request.onsuccess = () => resolve(request.result)
    request.onerror = () => reject(request.error)
  })

  return dbPromise
}

/**
 * Save a message to IndexedDB.
 */
export async function saveMessage(roomId, msg) {
  const db = await openDB()
  const tx = db.transaction('messages', 'readwrite')
  tx.objectStore('messages').put({
    roomId,
    msgId: msg.msgId,
    timestamp: msg.timestamp,
    data: msg,
  })
}

/**
 * Load cached messages for a room, ordered by timestamp.
 */
export async function loadMessages(roomId, limit = 100) {
  const db = await openDB()
  return new Promise((resolve, reject) => {
    const tx = db.transaction('messages', 'readonly')
    const store = tx.objectStore('messages')
    const index = store.index('byRoom')

    // Use a key range for this room
    const range = IDBKeyRange.bound([roomId, 0], [roomId, Number.MAX_SAFE_INTEGER])
    const results = []

    const request = index.openCursor(range, 'prev') // newest first
    request.onsuccess = (event) => {
      const cursor = event.target.result
      if (cursor && results.length < limit) {
        results.push(cursor.value.data)
        cursor.continue()
      } else {
        resolve(results.reverse()) // oldest first
      }
    }
    request.onerror = () => reject(request.error)
  })
}

/**
 * Delete all messages for a room.
 */
export async function deleteRoomMessages(roomId) {
  const db = await openDB()
  const tx = db.transaction('messages', 'readwrite')
  const store = tx.objectStore('messages')
  const index = store.index('byRoom')
  const range = IDBKeyRange.bound([roomId, 0], [roomId, Number.MAX_SAFE_INTEGER])

  const request = index.openCursor(range)
  request.onsuccess = (event) => {
    const cursor = event.target.result
    if (cursor) {
      cursor.delete()
      cursor.continue()
    }
  }
}

/**
 * Save a crypto key for a room.
 */
export async function saveCryptoKey(roomId, exportedKey) {
  const db = await openDB()
  const tx = db.transaction('crypto-keys', 'readwrite')
  tx.objectStore('crypto-keys').put({ roomId, key: exportedKey })
}

/**
 * Load a crypto key for a room.
 */
export async function loadCryptoKey(roomId) {
  const db = await openDB()
  return new Promise((resolve, reject) => {
    const tx = db.transaction('crypto-keys', 'readonly')
    const request = tx.objectStore('crypto-keys').get(roomId)
    request.onsuccess = () => resolve(request.result?.key ?? null)
    request.onerror = () => reject(request.error)
  })
}

/**
 * Save a TOFU key fingerprint.
 */
export async function saveTofuKey(id, data) {
  const db = await openDB()
  const tx = db.transaction('tofu-keys', 'readwrite')
  tx.objectStore('tofu-keys').put({ id, ...data })
}

/**
 * Load a TOFU key fingerprint.
 */
export async function loadTofuKey(id) {
  const db = await openDB()
  return new Promise((resolve, reject) => {
    const tx = db.transaction('tofu-keys', 'readonly')
    const request = tx.objectStore('tofu-keys').get(id)
    request.onsuccess = () => resolve(request.result ?? null)
    request.onerror = () => reject(request.error)
  })
}

/**
 * Save a signing key pair reference.
 */
export async function saveSigningKey(userId, data) {
  const db = await openDB()
  const tx = db.transaction('signing-keys', 'readwrite')
  tx.objectStore('signing-keys').put({ userId, ...data })
}

/**
 * Load a signing key pair reference.
 */
export async function loadSigningKey(userId) {
  const db = await openDB()
  return new Promise((resolve, reject) => {
    const tx = db.transaction('signing-keys', 'readonly')
    const request = tx.objectStore('signing-keys').get(userId)
    request.onsuccess = () => resolve(request.result ?? null)
    request.onerror = () => reject(request.error)
  })
}

/**
 * Export all data as JSON.
 */
export async function exportAll() {
  const db = await openDB()
  const data = { messages: [] }

  // Export messages only — crypto keys are excluded for security
  await new Promise((resolve, reject) => {
    const tx = db.transaction('messages', 'readonly')
    const request = tx.objectStore('messages').getAll()
    request.onsuccess = () => { data.messages = request.result; resolve() }
    request.onerror = () => reject(request.error)
  })

  return data
}

/**
 * Delete the entire IndexedDB database (for logout).
 */
export async function clearAllData() {
  // Close existing connection first
  if (dbPromise) {
    try {
      const db = await dbPromise
      db.close()
    } catch { /* ignore */ }
    dbPromise = null
  }

  return new Promise((resolve, reject) => {
    const request = indexedDB.deleteDatabase(DB_NAME)
    request.onsuccess = () => resolve()
    request.onerror = () => reject(request.error)
    request.onblocked = () => resolve() // still ok, will be deleted when unblocked
  })
}

/**
 * Import data from JSON.
 */
export async function importAll(json) {
  const db = await openDB()

  if (json.messages) {
    const tx = db.transaction('messages', 'readwrite')
    const store = tx.objectStore('messages')
    for (const msg of json.messages) {
      store.put(msg)
    }
  }

  if (json.cryptoKeys) {
    const tx = db.transaction('crypto-keys', 'readwrite')
    const store = tx.objectStore('crypto-keys')
    for (const key of json.cryptoKeys) {
      store.put(key)
    }
  }
}
