import { ref, watch, onUnmounted } from 'vue'
import { useClasp } from './useClasp.js'
import { useIdentity } from './useIdentity.js'
import { ADDR } from '../lib/constants.js'

/**
 * Per-room reactions composable.
 * Stores reactions at /chat/room/{rid}/reactions/{msgId}/{emoji}
 * Each key holds { users: [userId, ...] }
 */
export function useReactions(roomId) {
  const { connected, sessionId, subscribe, set } = useClasp()
  const { displayName } = useIdentity()

  // Map<messageId, Map<emoji, Set<userId>>>
  const reactions = ref(new Map())

  let unsub = null

  function start() {
    if (!roomId.value) return
    const rid = roomId.value

    unsub = subscribe(`${ADDR.ROOM}/${rid}/reactions/*`, (data, address) => {
      // address: /chat/room/{rid}/reactions/{msgId}/{emoji}
      const parts = address.split('/')
      const emoji = parts.pop()
      const msgId = parts.pop()

      if (data === null) {
        const msgReactions = reactions.value.get(msgId)
        if (msgReactions) {
          msgReactions.delete(emoji)
          if (msgReactions.size === 0) reactions.value.delete(msgId)
        }
      } else {
        if (!reactions.value.has(msgId)) {
          reactions.value.set(msgId, new Map())
        }
        reactions.value.get(msgId).set(emoji, new Set(data.users || []))
      }
      // Trigger reactivity
      reactions.value = new Map(reactions.value)
    })
  }

  function stop() {
    if (unsub) { unsub(); unsub = null }
    reactions.value = new Map()
  }

  function toggleReaction(messageId, emoji) {
    if (!connected.value || !roomId.value || !sessionId.value) return

    const rid = roomId.value
    const addr = `${ADDR.ROOM}/${rid}/reactions/${messageId}/${emoji}`
    const msgReactions = reactions.value.get(messageId)
    const currentUsers = msgReactions?.get(emoji) || new Set()

    const userId = sessionId.value
    const newUsers = new Set(currentUsers)

    if (newUsers.has(userId)) {
      newUsers.delete(userId)
    } else {
      newUsers.add(userId)
    }

    if (newUsers.size === 0) {
      set(addr, null)
    } else {
      set(addr, { users: Array.from(newUsers) })
    }
  }

  function getMessageReactions(messageId) {
    const msgReactions = reactions.value.get(messageId)
    if (!msgReactions) return []
    return Array.from(msgReactions.entries()).map(([emoji, users]) => ({
      emoji,
      count: users.size,
      active: users.has(sessionId.value),
    }))
  }

  watch(roomId, (newId, oldId) => {
    if (oldId) stop()
    if (newId) start()
  }, { immediate: true })

  onUnmounted(stop)

  return {
    reactions,
    toggleReaction,
    getMessageReactions,
  }
}
