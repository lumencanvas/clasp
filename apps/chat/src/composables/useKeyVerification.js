import { ref } from 'vue'
import { computeKeyFingerprint } from '../lib/crypto.js'
import { saveTofuKey, loadTofuKey } from '../lib/storage.js'

/**
 * TOFU (Trust On First Use) key verification.
 * Stores the first-seen fingerprint for each (roomId, userId) pair.
 * On subsequent encounters, compares against the stored fingerprint
 * and warns if changed.
 */

// Active key change warnings: Map<string, { userId, displayName, roomId, oldFingerprint, newFingerprint }>
const keyWarnings = ref(new Map())
// Dismissed warnings (acknowledged by user this session)
const dismissedWarnings = ref(new Set())

/**
 * Verify a peer's public key against stored TOFU fingerprint.
 * @param {string} roomId
 * @param {string} userId
 * @param {object} publicKeyJwk - The peer's ECDH public key in JWK format
 * @param {string} [displayName] - Optional display name for warning messages
 * @returns {{ trusted: boolean, firstSeen: boolean, changed: boolean, fingerprint: string }}
 */
async function verifyKey(roomId, userId, publicKeyJwk, displayName) {
  const fingerprint = await computeKeyFingerprint(publicKeyJwk)
  const id = `${roomId}:${userId}`

  try {
    const stored = await loadTofuKey(id)

    if (!stored) {
      // First time seeing this key — trust on first use
      await saveTofuKey(id, {
        fingerprint,
        firstSeen: Date.now(),
        userId,
        displayName: displayName || userId,
      })
      return { trusted: true, firstSeen: true, changed: false, fingerprint }
    }

    if (stored.fingerprint === fingerprint) {
      return { trusted: true, firstSeen: false, changed: false, fingerprint }
    }

    // Key has changed — emit warning
    const warningKey = `${roomId}:${userId}`
    keyWarnings.value = new Map(keyWarnings.value).set(warningKey, {
      userId,
      displayName: displayName || stored.displayName || userId,
      roomId,
      oldFingerprint: stored.fingerprint,
      newFingerprint: fingerprint,
      timestamp: Date.now(),
    })

    return { trusted: false, firstSeen: false, changed: true, fingerprint }
  } catch (e) {
    console.warn('[tofu] Failed to verify key:', e)
    return { trusted: false, firstSeen: false, changed: false, fingerprint }
  }
}

/**
 * Accept a changed key (update stored fingerprint and dismiss warning).
 */
async function acceptKeyChange(roomId, userId) {
  const warningKey = `${roomId}:${userId}`
  const warning = keyWarnings.value.get(warningKey)
  if (!warning) return

  const id = `${roomId}:${userId}`
  await saveTofuKey(id, {
    fingerprint: warning.newFingerprint,
    firstSeen: Date.now(),
    userId,
    displayName: warning.displayName,
  })

  const next = new Map(keyWarnings.value)
  next.delete(warningKey)
  keyWarnings.value = next
  dismissedWarnings.value = new Set(dismissedWarnings.value).add(warningKey)
}

/**
 * Dismiss a warning without updating the stored fingerprint.
 */
function dismissWarning(roomId, userId) {
  const warningKey = `${roomId}:${userId}`
  const next = new Map(keyWarnings.value)
  next.delete(warningKey)
  keyWarnings.value = next
  dismissedWarnings.value = new Set(dismissedWarnings.value).add(warningKey)
}

/**
 * Get the stored fingerprint for a peer (for safety number display).
 */
async function getStoredFingerprint(roomId, userId) {
  const id = `${roomId}:${userId}`
  try {
    const stored = await loadTofuKey(id)
    return stored?.fingerprint ?? null
  } catch {
    return null
  }
}

/**
 * Get active warnings for a specific room.
 */
function getRoomWarnings(roomId) {
  const warnings = []
  for (const [key, warning] of keyWarnings.value) {
    if (warning.roomId === roomId && !dismissedWarnings.value.has(key)) {
      warnings.push(warning)
    }
  }
  return warnings
}

export function useKeyVerification() {
  return {
    keyWarnings,
    verifyKey,
    acceptKeyChange,
    dismissWarning,
    getStoredFingerprint,
    getRoomWarnings,
  }
}
