import { ref, computed, readonly } from 'vue'
import { useClasp } from './useClasp.js'
import { useIdentity } from './useIdentity.js'
import { ADDR, ROOM_TYPES } from '../lib/constants.js'

const rooms = ref(new Map()) // roomId -> { id, name, type, isPublic, creatorId, creatorName, createdAt }
const joinedRoomIds = ref(new Set(JSON.parse(localStorage.getItem('clasp-chat-joined') || '[]')))
const currentRoomId = ref(null)
const discoveredRooms = ref([])
let unsubDiscovery = null

function persistJoined() {
  localStorage.setItem('clasp-chat-joined', JSON.stringify([...joinedRoomIds.value]))
}

const joinedRooms = computed(() => {
  return [...joinedRoomIds.value]
    .map(id => rooms.value.get(id))
    .filter(r => r && r.type !== ROOM_TYPES.DM)
    .sort((a, b) => a.name.localeCompare(b.name))
})

const dmRooms = computed(() => {
  return [...joinedRoomIds.value]
    .map(id => rooms.value.get(id))
    .filter(r => r && r.type === ROOM_TYPES.DM)
    .sort((a, b) => (b.createdAt || 0) - (a.createdAt || 0))
})

const currentRoom = computed(() => {
  if (!currentRoomId.value) return null
  return rooms.value.get(currentRoomId.value) || null
})

function createRoom({ name, type, isPublic, encrypted = false, passwordHash, passwordSalt }) {
  const { set, connected } = useClasp()
  const { userId, displayName } = useIdentity()
  if (!connected.value) return null

  const roomId = crypto.randomUUID()
  const roomData = {
    name,
    type,
    isPublic,
    creatorId: userId.value,
    creatorName: displayName.value,
    createdAt: Date.now(),
  }

  // Add password flag to registry if password-protected
  if (passwordHash) {
    roomData.hasPassword = true
  }

  // Register in global registry (public rooms only)
  if (isPublic) {
    set(`${ADDR.ROOM_REGISTRY}/${roomId}`, roomData)
  }
  // Set room meta
  const meta = { name, type, description: '', isPublic, encrypted }
  if (passwordHash) {
    meta.passwordHash = passwordHash
    meta.passwordSalt = passwordSalt
  }
  set(`${ADDR.ROOM}/${roomId}/meta`, meta)

  // Add locally
  rooms.value.set(roomId, { id: roomId, ...roomData })
  rooms.value = new Map(rooms.value) // trigger reactivity

  // Auto-join
  joinedRoomIds.value.add(roomId)
  joinedRoomIds.value = new Set(joinedRoomIds.value)
  persistJoined()

  return roomId
}

function joinRoom(roomId) {
  joinedRoomIds.value.add(roomId)
  joinedRoomIds.value = new Set(joinedRoomIds.value)
  persistJoined()

  // If we have discovered data, add to rooms map
  const discovered = discoveredRooms.value.find(r => r.id === roomId)
  if (discovered && !rooms.value.has(roomId)) {
    rooms.value.set(roomId, discovered)
    rooms.value = new Map(rooms.value)
  }
}

function leaveRoom(roomId) {
  joinedRoomIds.value.delete(roomId)
  joinedRoomIds.value = new Set(joinedRoomIds.value)
  persistJoined()

  if (currentRoomId.value === roomId) {
    // Switch to first available room or null
    const remaining = [...joinedRoomIds.value]
    currentRoomId.value = remaining.length > 0 ? remaining[0] : null
  }
}

function switchRoom(roomId) {
  currentRoomId.value = roomId
}

function deleteRoom(roomId) {
  const { set, connected } = useClasp()
  const { userId } = useIdentity()
  if (!connected.value) return

  const room = rooms.value.get(roomId)
  if (room && room.creatorId === userId.value) {
    set(`${ADDR.ROOM_REGISTRY}/${roomId}`, null)
    rooms.value.delete(roomId)
    rooms.value = new Map(rooms.value)
    leaveRoom(roomId)
  }
}

function createDM(targetUserId, targetName) {
  const { set, connected } = useClasp()
  const { userId, displayName } = useIdentity()
  if (!connected.value || !targetUserId) return null

  // Deterministic room ID from sorted user IDs
  const ids = [userId.value, targetUserId].sort()
  const roomId = `dm-${ids[0].slice(0, 8)}-${ids[1].slice(0, 8)}`

  // If already exists, just switch to it
  if (rooms.value.has(roomId)) {
    return roomId
  }

  const roomData = {
    name: targetName,
    type: ROOM_TYPES.DM,
    isPublic: false,
    creatorId: userId.value,
    creatorName: displayName.value,
    createdAt: Date.now(),
    dmUsers: {
      [userId.value]: displayName.value,
      [targetUserId]: targetName,
    },
  }

  // Set room meta (DMs skip the public registry)
  set(`${ADDR.ROOM}/${roomId}/meta`, {
    name: targetName,
    type: ROOM_TYPES.DM,
    isPublic: false,
    dmUsers: roomData.dmUsers,
  })

  // Add locally
  rooms.value.set(roomId, { id: roomId, ...roomData })
  rooms.value = new Map(rooms.value)

  // Auto-join
  joinedRoomIds.value.add(roomId)
  joinedRoomIds.value = new Set(joinedRoomIds.value)
  persistJoined()

  return roomId
}

function processRegistryEntry(data, roomId) {
  if (data === null) {
    // Room deleted
    discoveredRooms.value = discoveredRooms.value.filter(r => r.id !== roomId)
    rooms.value.delete(roomId)
    rooms.value = new Map(rooms.value)
    return
  }

  // Safety filter: skip non-public rooms (handles public->private toggles)
  if (!data.isPublic) {
    discoveredRooms.value = discoveredRooms.value.filter(r => r.id !== roomId)
    return
  }

  const roomData = { id: roomId, ...data }

  // Update rooms map
  rooms.value.set(roomId, roomData)
  rooms.value = new Map(rooms.value)

  // Update discovered list
  const idx = discoveredRooms.value.findIndex(r => r.id === roomId)
  if (idx >= 0) {
    discoveredRooms.value[idx] = roomData
  } else {
    discoveredRooms.value.push(roomData)
  }
  discoveredRooms.value = [...discoveredRooms.value]
}

function discoverPublicRooms() {
  const { subscribe, connected, client } = useClasp()
  if (!connected.value) return

  if (unsubDiscovery) {
    unsubDiscovery()
  }

  const registryPattern = `${ADDR.ROOM_REGISTRY}/*`

  unsubDiscovery = subscribe(registryPattern, (data, address) => {
    const roomId = address.split('/').pop()
    processRegistryEntry(data, roomId)
  })

  // Fix: process pre-existing state from CLASP snapshot cache.
  // The SNAPSHOT handler stores params but doesn't call notifySubscribers(),
  // so wildcard subscriptions never fire for already-set data.
  if (client.value?.params) {
    const prefix = `${ADDR.ROOM_REGISTRY}/`
    for (const [address, data] of client.value.params) {
      if (address.startsWith(prefix) && data !== null) {
        const roomId = address.slice(prefix.length)
        processRegistryEntry(data, roomId)
      }
    }
  }
}

function fetchRoomMeta(roomId) {
  const { subscribe, connected } = useClasp()
  if (!connected.value) return Promise.resolve(null)

  return new Promise((resolve) => {
    const unsub = subscribe(`${ADDR.ROOM}/${roomId}/meta`, (data) => {
      resolve(data)
      unsub()
    })
    setTimeout(() => {
      unsub()
      resolve(null)
    }, 3000)
  })
}

function updateRoomData(roomId, updates) {
  const existing = rooms.value.get(roomId)
  if (existing) {
    rooms.value.set(roomId, { ...existing, ...updates })
  } else {
    rooms.value.set(roomId, { id: roomId, ...updates })
  }
  rooms.value = new Map(rooms.value)
}

function stopDiscovery() {
  if (unsubDiscovery) {
    unsubDiscovery()
    unsubDiscovery = null
  }
}

export function useRooms() {
  return {
    rooms: readonly(rooms),
    joinedRoomIds: readonly(joinedRoomIds),
    joinedRooms,
    dmRooms,
    currentRoomId,
    currentRoom,
    discoveredRooms: readonly(discoveredRooms),
    createRoom,
    createDM,
    joinRoom,
    leaveRoom,
    switchRoom,
    deleteRoom,
    fetchRoomMeta,
    updateRoomData,
    discoverPublicRooms,
    stopDiscovery,
  }
}
