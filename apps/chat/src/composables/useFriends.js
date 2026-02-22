import { ref, computed } from 'vue'
import { useClasp } from './useClasp.js'
import { useIdentity } from './useIdentity.js'
import { useNotifications } from './useNotifications.js'
import { ADDR } from '../lib/constants.js'

/**
 * Friends composable.
 * Friends stored at: /chat/user/{userId}/friends/{friendId}
 * Friend requests persisted via SET: /chat/requests/{targetId}/{fromId}
 * Incoming requests subscribed at: /chat/requests/{myId}/* (wildcard)
 *
 * Two-step handshake for accepting:
 * 1. A accepts -> writes own side: SET /chat/user/A/friends/B
 * 2. A sends confirmation: SET /chat/requests/B/A { type: 'accepted', fromId: A }
 * 3. B receives confirmation -> writes own side: SET /chat/user/B/friends/A
 * 4. Both sides clean up: SET /chat/requests/{myId}/{peerId} null
 */

const friends = ref(new Map()) // friendId -> { name, avatarColor, status, addedAt }
const pendingRequests = ref([]) // [{ fromId, fromName, fromColor, timestamp }]
const showFriends = ref(false) // UI state: whether the friends panel is open

let unsubFriends = null
let unsubRequests = null
let initialLoadTimer = null
let initialized = false

function init() {
  if (initialized) return
  initialized = true

  const { subscribe, set, connected } = useClasp()
  const { userId } = useIdentity()
  const { addToast, notifyFriendRequest } = useNotifications()

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

  // Subscribe to incoming friend requests (wildcard: /chat/requests/{myId}/*)
  // Each request is a separate key: /chat/requests/{myId}/{fromId}
  // On reconnect, SNAPSHOT delivers all pending requests.
  // Suppress individual toasts during SNAPSHOT — show a single summary after.
  let isInitialLoad = true
  let initialRequestCount = 0
  pendingRequests.value = []

  unsubRequests = subscribe(`${ADDR.REQUESTS}/${userId.value}/*`, (data, address) => {
    const fromId = address.split('/').pop()

    // Null = cleanup deletion, remove from pending
    if (data === null) {
      pendingRequests.value = pendingRequests.value.filter(r => r.fromId !== fromId)
      return
    }

    if (!data || typeof data !== 'object') return

    // Handle accepted confirmation (two-step handshake step 3)
    if (data.type === 'accepted') {
      const { userId: myId } = useIdentity()
      // Write our side of the friendship
      set(`${ADDR.USER_PROFILE}/${myId.value}/friends/${data.fromId}`, {
        name: data.fromName,
        avatarColor: data.fromColor,
        addedAt: Date.now(),
      })
      // Clean up the accepted request from our inbox
      set(`${ADDR.REQUESTS}/${myId.value}/${data.fromId}`, null)
      return
    }

    // Don't add duplicate requests
    if (pendingRequests.value.some(r => r.fromId === fromId)) return

    // Don't show requests from people we're already friends with
    if (friends.value.has(fromId)) {
      set(`${ADDR.REQUESTS}/${userId.value}/${fromId}`, null)
      return
    }

    pendingRequests.value = [...pendingRequests.value, { ...data, fromId }]

    if (isInitialLoad) {
      initialRequestCount++
    } else {
      // Toast (with action to open friends panel) + system notification
      const name = data.fromName || fromId
      addToast(`Friend request from ${name}`, 'info', 5000, () => {
        showFriends.value = true
      })
      notifyFriendRequest(name)
    }
  })

  // After SNAPSHOT batch, show a single summary toast if there were pending requests
  initialLoadTimer = setTimeout(() => {
    initialLoadTimer = null
    if (initialRequestCount > 0) {
      const msg = initialRequestCount === 1
        ? '1 pending friend request'
        : `${initialRequestCount} pending friend requests`
      addToast(msg, 'info', 5000, () => {
        showFriends.value = true
      })
    }
    isInitialLoad = false
  }, 500)
}

function sendRequest(targetUserId, targetName) {
  const { set, connected } = useClasp()
  const { userId, displayName, avatarColor } = useIdentity()
  if (!connected.value) return

  // Don't friend yourself
  if (targetUserId === userId.value) return
  // Already friends
  if (friends.value.has(targetUserId)) return

  // SET persistent request: /chat/requests/{targetId}/{fromId}
  set(`${ADDR.REQUESTS}/${targetUserId}/${userId.value}`, {
    fromId: userId.value,
    fromName: displayName.value,
    fromColor: avatarColor.value,
    timestamp: Date.now(),
  })
}

function acceptRequest(fromId) {
  const { set, connected } = useClasp()
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

  // Step 2: Send confirmation to them (SET, persistent)
  set(`${ADDR.REQUESTS}/${fromId}/${userId.value}`, {
    type: 'accepted',
    fromId: userId.value,
    fromName: displayName.value,
    fromColor: avatarColor.value,
    timestamp: now,
  })

  // Step 3: Clean up the request from our inbox
  set(`${ADDR.REQUESTS}/${userId.value}/${fromId}`, null)

  // Remove from pending
  pendingRequests.value = pendingRequests.value.filter(r => r.fromId !== fromId)
}

function declineRequest(fromId) {
  const { set, connected } = useClasp()
  const { userId } = useIdentity()
  if (!connected.value) return

  // Delete the request from our inbox (persistent cleanup)
  set(`${ADDR.REQUESTS}/${userId.value}/${fromId}`, null)

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
  if (initialLoadTimer) { clearTimeout(initialLoadTimer); initialLoadTimer = null }
  friends.value = new Map()
  pendingRequests.value = []
  showFriends.value = false
  initialized = false
}

export function useFriends() {
  return {
    friends,
    friendList,
    pendingRequests,
    requestCount,
    showFriends,
    init,
    cleanup,
    sendRequest,
    acceptRequest,
    declineRequest,
    removeFriend,
    isFriend,
  }
}
