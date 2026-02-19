import { ref } from 'vue'
import { useClasp } from './useClasp.js'
import { useIdentity } from './useIdentity.js'
import { ADDR } from '../lib/constants.js'
import {
  generateSigningKeyPair,
  signData,
  verifySignature,
  importSigningPublicKey,
  exportKey,
} from '../lib/crypto.js'
import { saveSigningKey, loadSigningKey } from '../lib/storage.js'

/**
 * Message signing with ECDSA P-256.
 * Signs outgoing messages for cryptographic authentication.
 * Verifies incoming message signatures.
 */

let signingKeyPair = null
let signingPublicKeyJwk = null
let initPromise = null
// Cache of imported peer signing public keys: userId -> CryptoKey
const peerSigningKeys = new Map()
// In-flight peer key lookups: peerId -> Promise
const pendingKeyLookups = new Map()
const initialized = ref(false)

/**
 * Initialize signing key pair. Generates a new one if none exists.
 * Publishes public key to CLASP state.
 * Uses promise dedup to prevent concurrent initialization races.
 */
async function initSigning() {
  if (signingKeyPair) return
  if (initPromise) return initPromise

  initPromise = (async () => {
    try {
      const { userId } = useIdentity()
      const uid = userId.value
      if (!uid) return

      // Private key is non-extractable, so we regenerate each session
      signingKeyPair = await generateSigningKeyPair()
      signingPublicKeyJwk = await exportKey(signingKeyPair.publicKey)

      // Store public key reference
      await saveSigningKey(uid, { publicKeyJwk: signingPublicKeyJwk })

      // Publish signing public key to CLASP state
      const { set, connected } = useClasp()
      if (connected.value) {
        set(`${ADDR.USER_PROFILE}/${uid}/signingKey`, {
          publicKey: signingPublicKeyJwk,
          algorithm: 'ECDSA-P256-SHA256',
          timestamp: Date.now(),
        })
      }

      initialized.value = true
    } finally {
      initPromise = null
    }
  })()

  return initPromise
}

/**
 * Sign a message payload. Returns the base64 signature.
 * Signs the canonical form: { text, fromId, msgId, timestamp }
 */
async function signMessage(payload) {
  if (!signingKeyPair) await initSigning()
  if (!signingKeyPair) return null

  const canonical = JSON.stringify({
    text: payload.text,
    fromId: payload.fromId,
    msgId: payload.msgId,
    timestamp: payload.timestamp,
  })

  try {
    return await signData(signingKeyPair.privateKey, canonical)
  } catch (e) {
    console.warn('[signing] Failed to sign message:', e)
    return null
  }
}

/**
 * Get a peer's signing public key (from CLASP state or cache).
 */
async function getPeerSigningKey(peerId) {
  if (peerSigningKeys.has(peerId)) return peerSigningKeys.get(peerId)

  // Dedup concurrent lookups for the same peer
  if (pendingKeyLookups.has(peerId)) return pendingKeyLookups.get(peerId)

  const { subscribe } = useClasp()

  const promise = new Promise((resolve) => {
    let settled = false
    const unsub = subscribe(`${ADDR.USER_PROFILE}/${peerId}/signingKey`, async (data) => {
      if (settled) return
      settled = true
      unsub()
      pendingKeyLookups.delete(peerId)

      if (!data?.publicKey) {
        resolve(null)
        return
      }

      try {
        const key = await importSigningPublicKey(data.publicKey)
        peerSigningKeys.set(peerId, key)
        resolve(key)
      } catch (e) {
        console.warn(`[signing] Failed to import signing key for ${peerId}:`, e)
        resolve(null)
      }
    })

    // Timeout if no key available
    setTimeout(() => {
      if (settled) return
      settled = true
      unsub()
      pendingKeyLookups.delete(peerId)
      resolve(null)
    }, 2000)
  })

  pendingKeyLookups.set(peerId, promise)
  return promise
}

/**
 * Verify a message signature.
 * @returns {'verified' | 'failed' | 'unknown'}
 */
async function verifyMessage(payload) {
  if (!payload.signature) return 'unknown'

  // Need the sender's userId (not sessionId) to look up their signing key
  // The fromId in messages is sessionId; we need to map this
  // For now, use the userId field if available, otherwise return unknown
  const senderId = payload.userId || payload.fromId
  if (!senderId) return 'unknown'

  try {
    const peerKey = await getPeerSigningKey(senderId)
    if (!peerKey) return 'unknown'

    const canonical = JSON.stringify({
      text: payload.text,
      fromId: payload.fromId,
      msgId: payload.msgId,
      timestamp: payload.timestamp,
    })

    const valid = await verifySignature(peerKey, canonical, payload.signature)
    return valid ? 'verified' : 'failed'
  } catch (e) {
    console.warn('[signing] Verification error:', e)
    return 'unknown'
  }
}

export function useSigning() {
  return {
    initialized,
    initSigning,
    signMessage,
    verifyMessage,
  }
}
