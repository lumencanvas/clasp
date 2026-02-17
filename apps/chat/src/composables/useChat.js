import { ref, computed, watch, onUnmounted } from 'vue'
import { useClasp } from './useClasp.js'
import { useIdentity } from './useIdentity.js'
import { ADDR, TTL } from '../lib/constants.js'

/**
 * Per-room chat composable
 */
export function useChat(roomId) {
  const { connected, sessionId, subscribe, emit, set } = useClasp()
  const { userId, displayName, avatarColor } = useIdentity()

  const messages = ref([])
  const participants = ref(new Map())
  const typingUsers = ref(new Map())
  const isTyping = ref(false)

  let unsubMessages = null
  let unsubPresence = null
  let unsubTyping = null
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
      .filter(([id]) => id !== sessionId.value)
      .map(([, data]) => data.name)
  })

  const onlineCount = computed(() => participants.value.size)

  function joinChat() {
    if (!connected.value || !roomId.value) return

    const rid = roomId.value

    // Subscribe to messages (EMIT)
    unsubMessages = subscribe(`${ADDR.ROOM}/${rid}/messages`, (payload) => {
      if (!payload || typeof payload !== 'object') return
      // Skip own messages (already added optimistically)
      if (payload.fromId === sessionId.value) return

      messages.value.push({
        id: Date.now() + Math.random(),
        type: 'message',
        ...payload,
      })
    })

    // Subscribe to presence (SET with wildcard)
    unsubPresence = subscribe(`${ADDR.ROOM}/${rid}/presence/*`, (data, address) => {
      const uid = address.split('/').pop()

      if (data === null) {
        const user = participants.value.get(uid)
        if (user && uid !== sessionId.value) {
          addSystemMessage(`${user.name} left`)
        }
        participants.value.delete(uid)
        participants.value = new Map(participants.value)
      } else {
        const isNew = !participants.value.has(uid)
        participants.value.set(uid, { ...data, lastSeen: Date.now() })
        participants.value = new Map(participants.value)
        if (isNew && uid !== sessionId.value) {
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
    if (presenceInterval) { clearInterval(presenceInterval); presenceInterval = null }
    if (pruneInterval) { clearInterval(pruneInterval); pruneInterval = null }
    if (typingTimeout) { clearTimeout(typingTimeout); typingTimeout = null }

    // Clear our presence and typing
    if (connected.value && sessionId.value && rid) {
      set(`${ADDR.ROOM}/${rid}/presence/${sessionId.value}`, null)
      set(`${ADDR.ROOM}/${rid}/typing/${sessionId.value}`, null)
    }

    messages.value = []
    participants.value = new Map()
    typingUsers.value = new Map()
    isTyping.value = false
  }

  function sendMessage(text) {
    if (!connected.value || !text.trim() || !roomId.value) return

    const msgData = {
      from: displayName.value,
      fromId: sessionId.value,
      text: text.trim(),
      timestamp: Date.now(),
      avatarColor: avatarColor.value,
      type: 'text',
    }

    // Add optimistically
    messages.value.push({
      id: Date.now() + Math.random(),
      type: 'message',
      ...msgData,
    })

    // Emit to server
    emit(`${ADDR.ROOM}/${roomId.value}/messages`, msgData)

    // Clear typing
    stopTyping()
  }

  function handleTyping() {
    if (!connected.value || !sessionId.value || !roomId.value) return

    if (!isTyping.value) {
      isTyping.value = true
      set(`${ADDR.ROOM}/${roomId.value}/typing/${sessionId.value}`, {
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
      set(`${ADDR.ROOM}/${roomId.value}/typing/${sessionId.value}`, null)
    }
    clearTimeout(typingTimeout)
  }

  function announcePresence() {
    if (!connected.value || !sessionId.value || !roomId.value) return
    set(`${ADDR.ROOM}/${roomId.value}/presence/${sessionId.value}`, {
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
      if (uid === sessionId.value) continue
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
    sendMessage,
    handleTyping,
    leaveChat,
  }
}
