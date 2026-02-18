import { ref, computed, onUnmounted } from 'vue'
import { useClasp } from './useClasp.js'
import { useIdentity } from './useIdentity.js'
import { ADDR } from '../lib/constants.js'

/**
 * Friends composable.
 * Friends stored at: /chat/user/{userId}/friends/{friendId}
 * Friend requests sent via EMIT: /chat/requests/{targetId}
 * Incoming requests subscribed at: /chat/requests/{myId}
 *
 * Two-step handshake for accepting:
 * 1. A accepts -> writes own side: SET /chat/user/A/friends/B
 * 2. A sends confirmation: EMIT /chat/requests/B { type: 'accepted', fromId: A }
 * 3. B receives confirmation -> writes own side: SET /chat/user/B/friends/A
 */

const friends = ref(new Map()) // friendId -> { name, avatarColor, status, addedAt }
const pendingRequests = ref([]) // [{ fromId, fromName, fromColor, timestamp }]

let unsubFriends = null
let unsubRequests = null
let initialized = false

function init() {
  if (initialized) return
  initialized = true

  const { subscribe, connected } = useClasp()
  const { userId } = useIdentity()

  if (!connected.value || !userId.value) return

  // Subscribe to our friends list
  unsubFriends = subscribe(`${ADDR.USER_PROFILE}/${userId.value}/friends/*`, (data, address) => {
    const friendId = address.split('/').pop()
    if (data === null) {
      friends.value.delete(friendId)
    } else {
      friends.value.set(friendId, data)
    }
    friends.value = new Map(friends.value)
  })

  // Subscribe to incoming friend requests (new path: /chat/requests/{myId})
  unsubRequests = subscribe(`${ADDR.REQUESTS}/${userId.value}`, (data) => {
    if (!data || typeof data !== 'object') return

    // Handle accepted confirmation (two-step handshake step 3)
    if (data.type === 'accepted') {
      const { set } = useClasp()
      const { userId: myId } = useIdentity()
      // Write our side of the friendship
      set(`${ADDR.USER_PROFILE}/${myId.value}/friends/${data.fromId}`, {
        name: data.fromName,
        avatarColor: data.fromColor,
        addedAt: Date.now(),
      })
      return
    }

    // Don't add duplicate requests
    if (pendingRequests.value.some(r => r.fromId === data.fromId)) return
    pendingRequests.value = [...pendingRequests.value, data]
  })
}

function sendRequest(targetUserId, targetName) {
  const { emit, connected } = useClasp()
  const { userId, displayName, avatarColor } = useIdentity()
  if (!connected.value) return

  // Don't friend yourself
  if (targetUserId === userId.value) return
  // Already friends
  if (friends.value.has(targetUserId)) return

  emit(`${ADDR.REQUESTS}/${targetUserId}`, {
    fromId: userId.value,
    fromName: displayName.value,
    fromColor: avatarColor.value,
    timestamp: Date.now(),
  })
}

function acceptRequest(fromId) {
  const { set, emit: claspEmit, connected } = useClasp()
  const { userId, displayName, avatarColor } = useIdentity()
  if (!connected.value) return

  const request = pendingRequests.value.find(r => r.fromId === fromId)
  if (!request) return

  const now = Date.now()

  // Step 1: Add them to our friends (write own side)
  set(`${ADDR.USER_PROFILE}/${userId.value}/friends/${fromId}`, {
    name: request.fromName,
    avatarColor: request.fromColor,
    addedAt: now,
  })

  // Step 2: Send confirmation to them so they can write their side
  claspEmit(`${ADDR.REQUESTS}/${fromId}`, {
    type: 'accepted',
    fromId: userId.value,
    fromName: displayName.value,
    fromColor: avatarColor.value,
    timestamp: now,
  })

  // Remove from pending
  pendingRequests.value = pendingRequests.value.filter(r => r.fromId !== fromId)
}

function declineRequest(fromId) {
  pendingRequests.value = pendingRequests.value.filter(r => r.fromId !== fromId)
}

function removeFriend(friendId) {
  const { set, connected } = useClasp()
  const { userId } = useIdentity()
  if (!connected.value) return

  // Remove from our side only (under auth, can't write to other user's data)
  set(`${ADDR.USER_PROFILE}/${userId.value}/friends/${friendId}`, null)
}

function isFriend(targetUserId) {
  return friends.value.has(targetUserId)
}

const friendList = computed(() => {
  return Array.from(friends.value.entries())
    .map(([id, data]) => ({ id, ...data }))
    .sort((a, b) => (a.name || '').localeCompare(b.name || ''))
})

const requestCount = computed(() => pendingRequests.value.length)

function cleanup() {
  if (unsubFriends) { unsubFriends(); unsubFriends = null }
  if (unsubRequests) { unsubRequests(); unsubRequests = null }
  initialized = false
}

export function useFriends() {
  return {
    friends,
    friendList,
    pendingRequests,
    requestCount,
    init,
    cleanup,
    sendRequest,
    acceptRequest,
    declineRequest,
    removeFriend,
    isFriend,
  }
}
