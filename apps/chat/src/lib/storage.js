/**
 * IndexedDB wrapper for CLASP Chat persistent storage.
 * DB: clasp-chat, version 1
 * Stores: messages (keyPath: [roomId, msgId]), crypto-keys (keyPath: roomId)
 */

const DB_NAME = 'clasp-chat'
const DB_VERSION = 1

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
 * Export all data as JSON.
 */
export async function exportAll() {
  const db = await openDB()
  const data = { messages: [], cryptoKeys: [] }

  // Export messages
  await new Promise((resolve, reject) => {
    const tx = db.transaction('messages', 'readonly')
    const request = tx.objectStore('messages').getAll()
    request.onsuccess = () => { data.messages = request.result; resolve() }
    request.onerror = () => reject(request.error)
  })

  // Export crypto keys
  await new Promise((resolve, reject) => {
    const tx = db.transaction('crypto-keys', 'readonly')
    const request = tx.objectStore('crypto-keys').getAll()
    request.onsuccess = () => { data.cryptoKeys = request.result; resolve() }
    request.onerror = () => reject(request.error)
  })

  return data
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
