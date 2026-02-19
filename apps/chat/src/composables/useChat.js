import { ref, computed, watch, onUnmounted } from 'vue'
import { useClasp } from './useClasp.js'
import { useIdentity } from './useIdentity.js'
import { useNotifications } from './useNotifications.js'
import { ADDR, TTL } from '../lib/constants.js'
import { executeCommand, initBuiltinPlugins, getRegisteredCommands } from '../lib/plugins.js'
import { useStorage } from './useStorage.js'
import { useCrypto } from './useCrypto.js'

/**
 * Per-room chat composable
 * @param {Ref<string>} roomId
 * @param {Ref<boolean>} [isActive] - whether this room is currently visible
 */
export function useChat(roomId, isActive) {
  const { connected, sessionId, subscribe, emit, set } = useClasp()
  const { userId, displayName, avatarColor } = useIdentity()
  const { incrementUnread, notifyMessage } = useNotifications()
  const { loadCachedMessages, persistMessage } = useStorage()
  const { encrypt, decrypt, isEncrypted, loadRoomKey, subscribeKeyExchange, encryptedRooms, markPasswordProtected } = useCrypto()

  const messages = ref([])
  const participants = ref(new Map())
  const typingUsers = ref(new Map())
  const isTyping = ref(false)
  const replyTo = ref(null) // { id, from, text } or null
  const editingMessage = ref(null) // message object or null
  const isThrottled = ref(false)
  const waitingForKey = ref(false)

  // Rate limiting: max 5 messages per second
  const MSG_RATE_LIMIT = 5
  const MSG_RATE_WINDOW = 1000
  let messageTimes = []

  let unsubMessages = null
  let unsubPresence = null
  let unsubTyping = null
  let unsubCrypto = null
  let presenceInterval = null
  let typingTimeout = null
  let pruneInterval = null

  const sortedParticipants = computed(() => {
    return Array.from(participants.value.entries())
      .map(([id, data]) => ({ id, ...data }))
      .sort((a, b) => a.name.localeCompare(b.name))
  })

  const typingList = computed(() => {
    return Array.from(typingUsers.value.entries())
      .filter(([id]) => id !== userId.value)
      .map(([, data]) => data.name)
  })

  const onlineCount = computed(() => participants.value.size)

  async function handleIncomingMessage(payload) {
    if (!payload || typeof payload !== 'object') return

    // Decrypt if encrypted
    if (payload.encrypted && payload.iv) {
      const plaintext = await decrypt(roomId.value, payload.text, payload.iv)
      if (plaintext !== null) {
        payload = { ...payload, text: plaintext }
      } else if (isEncrypted(roomId.value)) {
        // Can't decrypt in encrypted room — show placeholder
        payload = { ...payload, text: '[encrypted message - key unavailable]' }
      }
      // Decrypt image if present
      if (payload.encryptedImage && payload.imageIv) {
        const imgPlain = await decrypt(roomId.value, payload.encryptedImage, payload.imageIv)
        if (imgPlain !== null) {
          payload = { ...payload, image: imgPlain, encryptedImage: undefined, imageIv: undefined }
        }
      }
    }

    // Handle edit messages
    if (payload.type === 'edit') {
      if (payload.fromId === sessionId.value) return
      // Decrypt edit text if encrypted
      let editText = payload.text
      if (payload.encrypted && payload.iv) {
        const plain = await decrypt(roomId.value, payload.text, payload.iv)
        if (plain !== null) editText = plain
      }
      const idx = messages.value.findIndex(m => m.msgId === payload.targetId)
      if (idx !== -1) {
        messages.value[idx] = { ...messages.value[idx], text: editText, edited: true }
      }
      return
    }

    // Handle delete messages
    if (payload.type === 'delete') {
      if (payload.fromId === sessionId.value) return
      const idx = messages.value.findIndex(m => m.msgId === payload.targetId)
      if (idx !== -1) {
        messages.value.splice(idx, 1)
      }
      return
    }

    // Skip own messages (already added optimistically)
    if (payload.fromId === sessionId.value) return

    const msgObj = {
      id: Date.now() + Math.random(),
      type: 'message',
      ...payload,
    }
    messages.value.push(msgObj)

    // Persist to IndexedDB
    persistMessage(roomId.value, payload)

    // Fire unread notification if this room is not active
    if (isActive && !isActive.value) {
      incrementUnread(roomId.value)
      notifyMessage(roomId.value, payload.from, payload.text || '')
    }
  }

  async function joinChat() {
    if (!connected.value || !roomId.value) return

    const rid = roomId.value

    // Load cached messages from IndexedDB
    try {
      const cached = await loadCachedMessages(rid)
      if (cached.length > 0) {
        const seen = new Set(messages.value.map(m => m.msgId).filter(Boolean))
        const toAdd = cached
          .filter(m => m.msgId && !seen.has(m.msgId))
          .map(m => ({ id: Date.now() + Math.random(), type: 'message', ...m }))
        messages.value = [...toAdd, ...messages.value]
      }
    } catch { /* ignore storage errors */ }

    // Subscribe to messages (EMIT)
    unsubMessages = subscribe(`${ADDR.ROOM}/${rid}/messages`, handleIncomingMessage)

    // Subscribe to presence (SET with wildcard)
    unsubPresence = subscribe(`${ADDR.ROOM}/${rid}/presence/*`, (data, address) => {
      const uid = address.split('/').pop()

      if (data === null) {
        const user = participants.value.get(uid)
        if (user && uid !== userId.value) {
          addSystemMessage(`${user.name} left`)
        }
        participants.value.delete(uid)
        participants.value = new Map(participants.value)
      } else {
        const isNew = !participants.value.has(uid)
        participants.value.set(uid, { ...data, lastSeen: Date.now() })
        participants.value = new Map(participants.value)
        if (isNew && uid !== userId.value) {
          addSystemMessage(`${data.name} joined`)
        }
      }
    })

    // Subscribe to typing indicators (SET with wildcard)
    unsubTyping = subscribe(`${ADDR.ROOM}/${rid}/typing/*`, (data, address) => {
      const uid = address.split('/').pop()
      if (data === null || data === false) {
        typingUsers.value.delete(uid)
      } else {
        typingUsers.value.set(uid, data)
        // Auto-expire after 3s
        setTimeout(() => {
          typingUsers.value.delete(uid)
          typingUsers.value = new Map(typingUsers.value)
        }, TTL.TYPING_EXPIRE)
      }
      typingUsers.value = new Map(typingUsers.value)
    })

    // Setup crypto key exchange for this room
    const existingKey = await loadRoomKey(rid).catch(e => {
      console.warn('[chat] Failed to load room key:', e)
      return null
    })

    // Subscribe to room meta for encrypted flag (3E: populate encryptedRooms on join)
    subscribe(`${ADDR.ROOM}/${rid}/meta`, (meta) => {
      if (meta && meta.encrypted) {
        encryptedRooms.value = new Set(encryptedRooms.value).add(rid)
        // If we don't have the key yet, mark as waiting
        if (!isEncrypted(rid) && !existingKey) {
          waitingForKey.value = true
        }
      }
      if (meta && meta.passwordHash) {
        markPasswordProtected(rid)
      }
    })

    // If room is already known as encrypted and we don't have the key, set waiting
    if (isEncrypted(rid) && !existingKey) {
      waitingForKey.value = true
    }

    unsubCrypto = subscribeKeyExchange(rid)

    // Announce presence
    announcePresence()
    presenceInterval = setInterval(() => {
      announcePresence()
    }, TTL.PRESENCE_HEARTBEAT)

    // Prune stale participants
    pruneInterval = setInterval(() => {
      pruneStale()
    }, TTL.PRESENCE_HEARTBEAT)

    addSystemMessage(`Joined #${rid}`)
  }

  function leaveChat() {
    const rid = roomId.value

    if (unsubMessages) { unsubMessages(); unsubMessages = null }
    if (unsubPresence) { unsubPresence(); unsubPresence = null }
    if (unsubTyping) { unsubTyping(); unsubTyping = null }
    if (unsubCrypto) { unsubCrypto(); unsubCrypto = null }
    if (presenceInterval) { clearInterval(presenceInterval); presenceInterval = null }
    if (pruneInterval) { clearInterval(pruneInterval); pruneInterval = null }
    if (typingTimeout) { clearTimeout(typingTimeout); typingTimeout = null }

    // Clear our presence and typing
    if (connected.value && userId.value && rid) {
      set(`${ADDR.ROOM}/${rid}/presence/${userId.value}`, null)
      set(`${ADDR.ROOM}/${rid}/typing/${userId.value}`, null)
    }

    messages.value = []
    participants.value = new Map()
    typingUsers.value = new Map()
    isTyping.value = false
    replyTo.value = null
    editingMessage.value = null
  }

  // Initialize plugins once
  let pluginsReady = null
  function ensurePlugins() {
    if (pluginsReady) return pluginsReady
    pluginsReady = initBuiltinPlugins({
      sendMessage: (t) => sendMessage(t),
      getCurrentRoom: () => roomId.value,
      getUser: () => ({ userId: userId.value, displayName: displayName.value }),
    })
    return pluginsReady
  }

  async function sendMessage(text, { image } = {}) {
    if (!connected.value || !roomId.value) return
    if (!text.trim() && !image) return

    // Block plaintext in encrypted rooms without key
    if (waitingForKey.value) {
      addSystemMessage('Waiting for room key... Cannot send messages yet.')
      return
    }

    // Client-side rate limiting
    const now = Date.now()
    messageTimes = messageTimes.filter(t => now - t < MSG_RATE_WINDOW)
    if (messageTimes.length >= MSG_RATE_LIMIT) {
      isThrottled.value = true
      setTimeout(() => { isThrottled.value = false }, MSG_RATE_WINDOW)
      addSystemMessage('Slow down! You are sending messages too fast.')
      return
    }
    messageTimes.push(now)

    // Route slash commands to plugin system
    if (text.startsWith('/') && !image) {
      await ensurePlugins()
      const handled = executeCommand(text, {
        sendMessage: (t) => sendMessage(t),
        roomId: roomId.value,
        userId: userId.value,
        displayName: displayName.value,
      })
      if (handled) {
        stopTyping()
        return
      }
    }

    const msgId = `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`

    const msgData = {
      from: displayName.value,
      fromId: sessionId.value,
      msgId,
      text: text.trim(),
      timestamp: Date.now(),
      avatarColor: avatarColor.value,
      type: 'text',
    }

    // Attach reply reference
    if (replyTo.value) {
      msgData.replyTo = {
        msgId: replyTo.value.msgId,
        from: replyTo.value.from,
        text: (replyTo.value.text || '').slice(0, 100),
      }
      replyTo.value = null
    }

    // Attach image data
    if (image) {
      msgData.image = image
      msgData.type = 'image'
    }

    // Add optimistically
    messages.value.push({
      id: Date.now() + Math.random(),
      type: 'message',
      ...msgData,
    })

    // Persist own message to IndexedDB
    persistMessage(roomId.value, msgData)

    // Encrypt if room is encrypted
    let emitData = msgData
    if (isEncrypted(roomId.value)) {
      const encryptedText = await encrypt(roomId.value, msgData.text)
      if (encryptedText) {
        emitData = { ...msgData, text: encryptedText.ciphertext, iv: encryptedText.iv, encrypted: true }
        // Encrypt image if present
        if (msgData.image) {
          const encryptedImg = await encrypt(roomId.value, msgData.image)
          if (encryptedImg) {
            emitData.encryptedImage = encryptedImg.ciphertext
            emitData.imageIv = encryptedImg.iv
            delete emitData.image
          }
        }
      }
    }

    // Emit to server
    emit(`${ADDR.ROOM}/${roomId.value}/messages`, emitData)

    // Clear typing
    stopTyping()
  }

  async function editMessage(msgId, newText) {
    if (!connected.value || !roomId.value || !newText.trim()) return

    // Update locally
    const idx = messages.value.findIndex(m => m.msgId === msgId)
    if (idx !== -1) {
      messages.value[idx] = { ...messages.value[idx], text: newText.trim(), edited: true }
    }

    // Build edit payload
    const editPayload = {
      type: 'edit',
      fromId: sessionId.value,
      targetId: msgId,
      text: newText.trim(),
      timestamp: Date.now(),
    }

    // Encrypt if room is encrypted
    if (isEncrypted(roomId.value)) {
      const encrypted = await encrypt(roomId.value, editPayload.text)
      if (encrypted) {
        editPayload.text = encrypted.ciphertext
        editPayload.iv = encrypted.iv
        editPayload.encrypted = true
      }
    }

    // Emit edit to peers
    emit(`${ADDR.ROOM}/${roomId.value}/messages`, editPayload)

    editingMessage.value = null
  }

  function deleteMessage(msgId) {
    if (!connected.value || !roomId.value) return

    // Remove locally
    const idx = messages.value.findIndex(m => m.msgId === msgId)
    if (idx !== -1) {
      messages.value.splice(idx, 1)
    }

    // Emit delete to peers
    emit(`${ADDR.ROOM}/${roomId.value}/messages`, {
      type: 'delete',
      fromId: sessionId.value,
      targetId: msgId,
      timestamp: Date.now(),
    })
  }

  function setReplyTo(message) {
    editingMessage.value = null
    replyTo.value = message ? { msgId: message.msgId, from: message.from, text: message.text } : null
  }

  function startEditing(message) {
    replyTo.value = null
    editingMessage.value = message
  }

  function cancelEditing() {
    editingMessage.value = null
  }

  function handleTyping() {
    if (!connected.value || !sessionId.value || !roomId.value) return

    if (!isTyping.value) {
      isTyping.value = true
      set(`${ADDR.ROOM}/${roomId.value}/typing/${userId.value}`, {
        name: displayName.value,
        timestamp: Date.now(),
      })
    }

    clearTimeout(typingTimeout)
    typingTimeout = setTimeout(() => {
      stopTyping()
    }, TTL.TYPING_TIMEOUT)
  }

  function stopTyping() {
    if (isTyping.value && connected.value && sessionId.value && roomId.value) {
      isTyping.value = false
      set(`${ADDR.ROOM}/${roomId.value}/typing/${userId.value}`, null)
    }
    clearTimeout(typingTimeout)
  }

  function announcePresence() {
    if (!connected.value || !userId.value || !roomId.value) return
    set(`${ADDR.ROOM}/${roomId.value}/presence/${userId.value}`, {
      name: displayName.value,
      avatarColor: avatarColor.value,
      joinedAt: Date.now(),
      lastSeen: Date.now(),
    })
  }

  function pruneStale() {
    const now = Date.now()
    let changed = false
    for (const [uid, data] of participants.value) {
      if (uid === userId.value) continue
      if (now - (data.lastSeen || data.joinedAt || 0) > TTL.PRESENCE_STALE) {
        participants.value.delete(uid)
        changed = true
      }
    }
    if (changed) {
      participants.value = new Map(participants.value)
    }
  }

  function addSystemMessage(text) {
    messages.value.push({
      id: Date.now() + Math.random(),
      type: 'system',
      text,
      timestamp: Date.now(),
    })
  }

  // Watch for room changes
  watch(roomId, (newId, oldId) => {
    if (oldId) leaveChat()
    if (newId) joinChat()
  }, { immediate: true })

  // When key arrives for this room, clear waitingForKey
  watch(encryptedRooms, (rooms) => {
    if (roomId.value && rooms.has(roomId.value) && waitingForKey.value) {
      // Check if we actually have the key now (isEncrypted checks the roomKeys map)
      if (isEncrypted(roomId.value)) {
        waitingForKey.value = false
        addSystemMessage('Room key received. E2E encryption active.')
      }
    }
  }, { deep: true })

  // Cleanup
  onUnmounted(() => {
    leaveChat()
  })

  return {
    messages,
    participants,
    sortedParticipants,
    typingUsers,
    typingList,
    onlineCount,
    replyTo,
    editingMessage,
    sendMessage,
    editMessage,
    deleteMessage,
    setReplyTo,
    startEditing,
    cancelEditing,
    handleTyping,
    leaveChat,
    isThrottled,
    waitingForKey,
    async ensurePluginsReady() { await ensurePlugins() },
    getRegisteredCommands,
  }
}
