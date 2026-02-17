import { ref, computed, readonly } from 'vue'
import { useClasp } from './useClasp.js'
import { useIdentity } from './useIdentity.js'
import { ADDR } from '../lib/constants.js'
import { generateId } from '../lib/utils.js'

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
    .filter(Boolean)
    .sort((a, b) => a.name.localeCompare(b.name))
})

const currentRoom = computed(() => {
  if (!currentRoomId.value) return null
  return rooms.value.get(currentRoomId.value) || null
})

function createRoom({ name, type, isPublic }) {
  const { set, connected } = useClasp()
  const { userId, displayName } = useIdentity()
  if (!connected.value) return null

  const roomId = generateId(8)
  const roomData = {
    name,
    type,
    isPublic,
    creatorId: userId.value,
    creatorName: displayName.value,
    createdAt: Date.now(),
  }

  // Register in global registry
  set(`${ADDR.ROOM_REGISTRY}/${roomId}`, roomData)
  // Set room meta
  set(`${ADDR.ROOM}/${roomId}/meta`, { name, type, description: '', isPublic })

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

function discoverPublicRooms() {
  const { subscribe, connected } = useClasp()
  if (!connected.value) return

  if (unsubDiscovery) {
    unsubDiscovery()
  }

  unsubDiscovery = subscribe(`${ADDR.ROOM_REGISTRY}/*`, (data, address) => {
    const roomId = address.split('/').pop()

    if (data === null) {
      // Room deleted
      discoveredRooms.value = discoveredRooms.value.filter(r => r.id !== roomId)
      rooms.value.delete(roomId)
      rooms.value = new Map(rooms.value)
      return
    }

    const roomData = { id: roomId, ...data }

    // Update rooms map
    rooms.value.set(roomId, roomData)
    rooms.value = new Map(rooms.value)

    // Update discovered list (only public rooms)
    if (data.isPublic) {
      const idx = discoveredRooms.value.findIndex(r => r.id === roomId)
      if (idx >= 0) {
        discoveredRooms.value[idx] = roomData
      } else {
        discoveredRooms.value.push(roomData)
      }
      discoveredRooms.value = [...discoveredRooms.value]
    }
  })
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
    currentRoomId,
    currentRoom,
    discoveredRooms: readonly(discoveredRooms),
    createRoom,
    joinRoom,
    leaveRoom,
    switchRoom,
    deleteRoom,
    discoverPublicRooms,
    stopDiscovery,
  }
}
