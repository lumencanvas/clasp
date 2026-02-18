import { ref, computed } from 'vue'
import { useClasp } from './useClasp.js'
import { useIdentity } from './useIdentity.js'
import { useRooms } from './useRooms.js'
import { ADDR } from '../lib/constants.js'

/**
 * Admin tools composable.
 * Room creator can kick/ban members and mod-delete messages.
 */
export function useAdmin(roomId) {
  const { connected, set, emit: claspEmit, subscribe } = useClasp()
  const { userId } = useIdentity()
  const { currentRoom } = useRooms()

  const bannedUsers = ref(new Set())

  const isRoomCreator = computed(() => {
    return currentRoom.value?.creatorId === userId.value
  })

  /**
   * Kick a user from the room (clear their presence).
   */
  function kickUser(targetUserId) {
    if (!connected.value || !roomId.value) return
    const rid = roomId.value

    // Clear their presence
    set(`${ADDR.ROOM}/${rid}/presence/${targetUserId}`, null)

    // Emit kick notice
    claspEmit(`${ADDR.ROOM}/${rid}/messages`, {
      type: 'text',
      fromId: userId.value,
      from: 'System',
      msgId: `kick-${Date.now()}`,
      text: `User was kicked from the room.`,
      timestamp: Date.now(),
      avatarColor: '#6b7280',
    })
  }

  /**
   * Ban a user from the room.
   */
  function banUser(targetUserId) {
    if (!connected.value || !roomId.value) return
    const rid = roomId.value

    // Set ban record
    set(`${ADDR.ROOM}/${rid}/bans/${targetUserId}`, {
      bannedBy: userId.value,
      timestamp: Date.now(),
    })

    // Also kick them
    kickUser(targetUserId)

    bannedUsers.value = new Set(bannedUsers.value).add(targetUserId)
  }

  /**
   * Delete a message as admin/mod.
   */
  function modDeleteMessage(msgId) {
    if (!connected.value || !roomId.value) return

    claspEmit(`${ADDR.ROOM}/${roomId.value}/messages`, {
      type: 'delete',
      fromId: userId.value,
      targetId: msgId,
      timestamp: Date.now(),
      isModAction: true,
    })
  }

  /**
   * Subscribe to ban list for this room.
   */
  function subscribeBans() {
    if (!roomId.value) return () => {}
    return subscribe(`${ADDR.ROOM}/${roomId.value}/bans/*`, (data, address) => {
      const targetId = address.split('/').pop()
      if (data === null) {
        bannedUsers.value.delete(targetId)
      } else {
        bannedUsers.value.add(targetId)
      }
      bannedUsers.value = new Set(bannedUsers.value)
    })
  }

  function isUserBanned(targetUserId) {
    return bannedUsers.value.has(targetUserId)
  }

  return {
    isRoomCreator,
    bannedUsers,
    kickUser,
    banUser,
    modDeleteMessage,
    subscribeBans,
    isUserBanned,
  }
}
