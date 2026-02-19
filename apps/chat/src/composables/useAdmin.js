import { ref, computed } from 'vue'
import { useClasp } from './useClasp.js'
import { useIdentity } from './useIdentity.js'
import { useRooms } from './useRooms.js'
import { useCrypto } from './useCrypto.js'
import { ADDR } from '../lib/constants.js'

/**
 * Admin tools composable.
 * Room creator and promoted admins can kick/ban members and mod-delete messages.
 * Only the creator can promote/demote admins and delete the room.
 */
export function useAdmin(roomId) {
  const { connected, set, emit: claspEmit, subscribe } = useClasp()
  const { userId } = useIdentity()
  const { currentRoom } = useRooms()
  const { isEncrypted, rotateRoomKey } = useCrypto()

  const bannedUsers = ref(new Set())
  const adminList = ref(new Map()) // userId -> { role, promotedBy, timestamp }

  const isRoomCreator = computed(() => {
    return currentRoom.value?.creatorId === userId.value
  })

  const isAdmin = computed(() => {
    return isRoomCreator.value || adminList.value.has(userId.value)
  })

  /**
   * Subscribe to admin list for this room.
   */
  function subscribeAdmins() {
    if (!roomId.value) return () => {}
    return subscribe(`${ADDR.ROOM}/${roomId.value}/admin/*`, (data, address) => {
      const targetId = address.split('/').pop()
      if (data === null) {
        adminList.value.delete(targetId)
      } else {
        adminList.value.set(targetId, data)
      }
      adminList.value = new Map(adminList.value)
    })
  }

  /**
   * Promote a user to admin (creator-only).
   */
  function promoteToAdmin(targetUserId) {
    if (!connected.value || !roomId.value || !isRoomCreator.value) return
    set(`${ADDR.ROOM}/${roomId.value}/admin/${targetUserId}`, {
      role: 'admin',
      promotedBy: userId.value,
      timestamp: Date.now(),
    })
  }

  /**
   * Demote an admin (creator-only).
   */
  function demoteAdmin(targetUserId) {
    if (!connected.value || !roomId.value || !isRoomCreator.value) return
    set(`${ADDR.ROOM}/${roomId.value}/admin/${targetUserId}`, null)
  }

  /**
   * Check if a given user is an admin or the creator.
   */
  function isUserAdmin(targetUserId) {
    return targetUserId === currentRoom.value?.creatorId || adminList.value.has(targetUserId)
  }

  /**
   * Kick a user from the room (clear their presence).
   */
  function kickUser(targetUserId) {
    if (!connected.value || !roomId.value || !isAdmin.value) return
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
    if (!connected.value || !roomId.value || !isAdmin.value) return
    const rid = roomId.value

    // Set ban record
    set(`${ADDR.ROOM}/${rid}/bans/${targetUserId}`, {
      bannedBy: userId.value,
      timestamp: Date.now(),
    })

    // Also kick them
    kickUser(targetUserId)

    bannedUsers.value = new Set(bannedUsers.value).add(targetUserId)

    // Rotate room key if encrypted (so banned user can't decrypt new messages)
    if (isEncrypted(rid)) {
      rotateRoomKey(rid).catch(e => console.warn('[admin] Key rotation failed:', e))
    }
  }

  /**
   * Delete a message as admin/mod.
   */
  function modDeleteMessage(msgId) {
    if (!connected.value || !roomId.value || !isAdmin.value) return

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
    isAdmin,
    adminList,
    bannedUsers,
    subscribeAdmins,
    promoteToAdmin,
    demoteAdmin,
    isUserAdmin,
    kickUser,
    banUser,
    modDeleteMessage,
    subscribeBans,
    isUserBanned,
  }
}
