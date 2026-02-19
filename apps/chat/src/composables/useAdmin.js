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
  const { currentRoom, updateRoomData } = useRooms()
  const { isEncrypted, rotateRoomKey, removePeerKey } = useCrypto()

  const bannedUsers = ref(new Map()) // userId -> { id, name, bannedBy, timestamp }
  const adminList = ref(new Map()) // userId -> { role, promotedBy, timestamp }
  const roomMeta = ref(null)

  const isRoomCreator = computed(() => {
    return currentRoom.value?.creatorId === userId.value
  })

  const isAdmin = computed(() => {
    return isRoomCreator.value || adminList.value.has(userId.value)
  })

  const bannedUsersList = computed(() => {
    return [...bannedUsers.value.values()].sort((a, b) => (b.timestamp || 0) - (a.timestamp || 0))
  })

  /**
   * Subscribe to room meta for this room.
   */
  function subscribeRoomMeta() {
    if (!roomId.value) return () => {}
    return subscribe(`${ADDR.ROOM}/${roomId.value}/meta`, (data) => {
      roomMeta.value = data
    })
  }

  /**
   * Update room name (admin). Also updates registry if public.
   */
  function updateRoomName(newName) {
    if (!connected.value || !roomId.value || !isAdmin.value || !newName?.trim()) return
    const rid = roomId.value
    const meta = roomMeta.value || {}
    const updatedMeta = { ...meta, name: newName.trim() }
    set(`${ADDR.ROOM}/${rid}/meta`, updatedMeta)

    // Update registry if public
    if (updatedMeta.isPublic) {
      const room = currentRoom.value
      if (room) {
        set(`${ADDR.ROOM_REGISTRY}/${rid}`, { ...room, name: newName.trim() })
      }
    }

    // Update local rooms map
    updateRoomData(rid, { name: newName.trim() })
  }

  /**
   * Toggle room public/private (creator-only).
   */
  function togglePublic(makePublic) {
    if (!connected.value || !roomId.value || !isRoomCreator.value) return
    const rid = roomId.value
    const meta = roomMeta.value || {}
    const updatedMeta = { ...meta, isPublic: makePublic }
    set(`${ADDR.ROOM}/${rid}/meta`, updatedMeta)

    if (makePublic) {
      // Write to registry
      const room = currentRoom.value || {}
      set(`${ADDR.ROOM_REGISTRY}/${rid}`, {
        name: updatedMeta.name || room.name,
        type: updatedMeta.type || room.type,
        isPublic: true,
        creatorId: room.creatorId,
        creatorName: room.creatorName,
        createdAt: room.createdAt,
        hasPassword: !!updatedMeta.passwordHash,
      })
    } else {
      // Remove from registry
      set(`${ADDR.ROOM_REGISTRY}/${rid}`, null)
    }

    // Update local rooms map
    updateRoomData(rid, { isPublic: makePublic })
  }

  /**
   * Update or remove room password (creator-only).
   */
  function updatePassword(hash, salt) {
    if (!connected.value || !roomId.value || !isRoomCreator.value) return
    const rid = roomId.value
    const meta = roomMeta.value || {}
    const updatedMeta = { ...meta }
    if (hash && salt) {
      updatedMeta.passwordHash = hash
      updatedMeta.passwordSalt = salt
    } else {
      delete updatedMeta.passwordHash
      delete updatedMeta.passwordSalt
    }
    set(`${ADDR.ROOM}/${rid}/meta`, updatedMeta)

    // Update registry hasPassword flag if public
    if (updatedMeta.isPublic) {
      const room = currentRoom.value || {}
      set(`${ADDR.ROOM_REGISTRY}/${rid}`, {
        ...room,
        hasPassword: !!(hash && salt),
      })
    }
  }

  /**
   * Unban a user from the room.
   */
  function unbanUser(targetUserId) {
    if (!connected.value || !roomId.value || !isAdmin.value) return
    set(`${ADDR.ROOM}/${roomId.value}/bans/${targetUserId}`, null)
    bannedUsers.value.delete(targetUserId)
    bannedUsers.value = new Map(bannedUsers.value)
  }

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
  function banUser(targetUserId, targetName) {
    if (!connected.value || !roomId.value || !isAdmin.value) return
    const rid = roomId.value

    const banRecord = {
      bannedBy: userId.value,
      timestamp: Date.now(),
      name: targetName || 'Unknown',
    }

    // Set ban record
    set(`${ADDR.ROOM}/${rid}/bans/${targetUserId}`, banRecord)

    // Also kick them
    kickUser(targetUserId)

    bannedUsers.value.set(targetUserId, { id: targetUserId, ...banRecord })
    bannedUsers.value = new Map(bannedUsers.value)

    // Rotate room key if encrypted (so banned user can't decrypt new messages)
    if (isEncrypted(rid)) {
      removePeerKey(rid, targetUserId)
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
        bannedUsers.value.set(targetId, { id: targetId, ...data })
      }
      bannedUsers.value = new Map(bannedUsers.value)
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
    bannedUsersList,
    roomMeta,
    subscribeAdmins,
    subscribeRoomMeta,
    promoteToAdmin,
    demoteAdmin,
    isUserAdmin,
    kickUser,
    banUser,
    modDeleteMessage,
    subscribeBans,
    isUserBanned,
    updateRoomName,
    togglePublic,
    updatePassword,
    unbanUser,
  }
}
