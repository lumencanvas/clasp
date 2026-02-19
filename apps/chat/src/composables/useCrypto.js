import { ref } from 'vue'
import { useClasp } from './useClasp.js'
import { useIdentity } from './useIdentity.js'
import { ADDR } from '../lib/constants.js'
import {
  generateRoomKey,
  generateECDHKeyPair,
  deriveSharedSecret,
  encryptMessage,
  decryptMessage,
  exportKey,
  importKey,
} from '../lib/crypto.js'
import { saveCryptoKey, loadCryptoKey } from '../lib/storage.js'

/**
 * Per-room E2E encryption key management.
 * Keys stored in-memory + persisted to IndexedDB.
 */

// roomId -> CryptoKey
const roomKeys = new Map()
const encryptedRooms = ref(new Set())
// Rooms that require password proof for key exchange
const passwordRooms = new Set()

// ECDH key pair for this session (generated lazily)
let ecdhKeyPair = null

async function getECDHKeyPair() {
  if (!ecdhKeyPair) {
    ecdhKeyPair = await generateECDHKeyPair()
  }
  return ecdhKeyPair
}

/**
 * Initialize encryption for a room. Generates a new AES key and stores it.
 */
async function enableEncryption(roomId) {
  const key = await generateRoomKey()
  roomKeys.set(roomId, key)
  encryptedRooms.value = new Set(encryptedRooms.value).add(roomId)

  // Persist to IndexedDB
  const exported = await exportKey(key)
  await saveCryptoKey(roomId, exported)

  // Publish our public key for key exchange
  await publishPublicKey(roomId)

  return key
}

/**
 * Load a persisted room key from IndexedDB.
 */
async function loadRoomKey(roomId) {
  if (roomKeys.has(roomId)) return roomKeys.get(roomId)

  try {
    const exported = await loadCryptoKey(roomId)
    if (!exported) return null

    const key = await importKey(exported, 'aes')
    roomKeys.set(roomId, key)
    encryptedRooms.value = new Set(encryptedRooms.value).add(roomId)
    return key
  } catch (e) {
    console.warn(`[crypto] Failed to load room key for ${roomId}:`, e)
    return null
  }
}

/**
 * Publish our ECDH public key for key exchange.
 */
async function publishPublicKey(roomId) {
  const { set, connected } = useClasp()
  const { userId } = useIdentity()
  if (!connected.value) return

  const kp = await getECDHKeyPair()
  const pubJwk = await exportKey(kp.publicKey)

  set(`${ADDR.ROOM}/${roomId}/crypto/pubkey/${userId.value}`, {
    publicKey: pubJwk,
    timestamp: Date.now(),
  })
}

/**
 * Subscribe to key exchange events for a room.
 * When a new member publishes their public key, derive a shared secret
 * and send them the room key encrypted with it.
 */
function subscribeKeyExchange(roomId) {
  const { subscribe, emit: claspEmit, connected } = useClasp()
  const { userId } = useIdentity()

  // Watch for new public keys
  const unsubPubkey = subscribe(`${ADDR.ROOM}/${roomId}/crypto/pubkey/*`, async (data, address) => {
    if (!data || !connected.value) return
    const peerId = address.split('/').pop()
    if (peerId === userId.value) return

    const roomKey = roomKeys.get(roomId)
    if (!roomKey) return // We don't have the room key

    // If room is password-protected, verify peer has published a valid proof
    if (passwordRooms.has(roomId)) {
      const proof = await new Promise((resolve) => {
        const unsub = subscribe(`${ADDR.ROOM}/${roomId}/crypto/proof/${peerId}`, (proofData) => {
          resolve(proofData)
          unsub()
        })
        setTimeout(() => resolve(null), 2000)
      })
      if (!proof || !proof.hash) {
        console.warn(`[crypto] Peer ${peerId} has no password proof, skipping key exchange`)
        return
      }
    }

    try {
      // Derive shared secret with peer's public key
      const peerPubKey = await importKey(data.publicKey, 'ecdh-public')
      const kp = await getECDHKeyPair()
      const sharedKey = await deriveSharedSecret(kp.privateKey, peerPubKey)

      // Encrypt the room key with the shared secret
      const roomKeyJwk = JSON.stringify(await exportKey(roomKey))
      const encrypted = await encryptMessage(sharedKey, roomKeyJwk)

      // Send the encrypted room key to the peer
      claspEmit(`${ADDR.ROOM}/${roomId}/crypto/keyex/${peerId}`, {
        fromId: userId.value,
        encryptedKey: encrypted.ciphertext,
        iv: encrypted.iv,
        senderPublicKey: await exportKey(kp.publicKey),
      })
    } catch (e) {
      console.warn(`[crypto] Key exchange failed for peer ${peerId}:`, e)
    }
  })

  // Watch for encrypted room keys sent to us
  const unsubKeyex = subscribe(`${ADDR.ROOM}/${roomId}/crypto/keyex/${userId.value}`, async (data) => {
    if (!data || roomKeys.has(roomId)) return

    try {
      // Derive shared secret with sender's public key
      const senderPubKey = await importKey(data.senderPublicKey, 'ecdh-public')
      const kp = await getECDHKeyPair()
      const sharedKey = await deriveSharedSecret(kp.privateKey, senderPubKey)

      // Decrypt the room key
      const roomKeyJwk = await decryptMessage(sharedKey, data.encryptedKey, data.iv)
      const roomKey = await importKey(JSON.parse(roomKeyJwk), 'aes')

      roomKeys.set(roomId, roomKey)
      encryptedRooms.value = new Set(encryptedRooms.value).add(roomId)

      // Persist
      const exported = await exportKey(roomKey)
      await saveCryptoKey(roomId, exported)
    } catch (e) {
      console.warn(`[crypto] Failed to receive room key:`, e)
    }
  })

  return () => {
    unsubPubkey()
    unsubKeyex()
  }
}

/**
 * Encrypt a message text for a room. Returns { ciphertext, iv } or null if not encrypted.
 */
async function encrypt(roomId, text) {
  const key = roomKeys.get(roomId)
  if (!key) return null
  return encryptMessage(key, text)
}

/**
 * Decrypt a message for a room. Returns plaintext or null on failure.
 */
async function decrypt(roomId, ciphertext, iv) {
  const key = roomKeys.get(roomId) || await loadRoomKey(roomId)
  if (!key) return null
  try {
    return await decryptMessage(key, ciphertext, iv)
  } catch {
    return null
  }
}

/**
 * Check if a room has encryption enabled.
 */
function isEncrypted(roomId) {
  return encryptedRooms.value.has(roomId)
}

/**
 * Rotate the room key after a member is removed.
 * Generates a new AES key, publishes new public key to trigger re-exchange.
 */
async function rotateRoomKey(roomId) {
  if (!roomKeys.has(roomId)) return

  // Generate new key
  const newKey = await generateRoomKey()
  roomKeys.set(roomId, newKey)
  encryptedRooms.value = new Set(encryptedRooms.value).add(roomId)

  // Persist to IndexedDB
  const exported = await exportKey(newKey)
  await saveCryptoKey(roomId, exported)

  // Re-publish our public key to trigger key exchange with remaining members
  await publishPublicKey(roomId)
}

/**
 * Mark a room as password-protected for key exchange gating.
 */
function markPasswordProtected(roomId) {
  passwordRooms.add(roomId)
}

export function useCrypto() {
  return {
    encryptedRooms,
    enableEncryption,
    loadRoomKey,
    subscribeKeyExchange,
    publishPublicKey,
    encrypt,
    decrypt,
    isEncrypted,
    rotateRoomKey,
    markPasswordProtected,
  }
}
