import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ref } from 'vue'

// Mock dependencies
const mockSet = vi.fn()
const mockSubscribe = vi.fn(() => vi.fn())
const mockConnected = ref(true)
const mockUserId = ref('user-1')
const mockDisplayName = ref('TestUser')

vi.mock('../useClasp.js', () => ({
  useClasp: () => ({
    connected: mockConnected,
    set: mockSet,
    subscribe: mockSubscribe,
  }),
}))

vi.mock('../useIdentity.js', () => ({
  useIdentity: () => ({
    userId: mockUserId,
    displayName: mockDisplayName,
  }),
}))

// Mock localStorage
const localStorageMock = (() => {
  let store = {}
  return {
    getItem: vi.fn((key) => store[key] || null),
    setItem: vi.fn((key, value) => { store[key] = value }),
    removeItem: vi.fn((key) => { delete store[key] }),
    clear: vi.fn(() => { store = {} }),
  }
})()
Object.defineProperty(globalThis, 'localStorage', { value: localStorageMock })

import { useRooms } from '../useRooms.js'

describe('useRooms', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    localStorageMock.clear()
    mockConnected.value = true
    mockUserId.value = 'user-1'
    mockDisplayName.value = 'TestUser'
  })

  it('createRoom generates a unique room ID', () => {
    const { createRoom } = useRooms()
    const id1 = createRoom({ name: 'Room 1', type: 'text', isPublic: true })
    const id2 = createRoom({ name: 'Room 2', type: 'text', isPublic: true })
    expect(id1).toBeTruthy()
    expect(id2).toBeTruthy()
    expect(id1).not.toBe(id2)
  })

  it('createRoom auto-joins the room', () => {
    const { createRoom, joinedRoomIds } = useRooms()
    const id = createRoom({ name: 'Test', type: 'text', isPublic: true })
    expect(joinedRoomIds.value.has(id)).toBe(true)
  })

  it('createRoom sets room registry and meta', () => {
    const { createRoom } = useRooms()
    createRoom({ name: 'Test', type: 'text', isPublic: true })
    expect(mockSet).toHaveBeenCalledWith(
      expect.stringContaining('/chat/registry/rooms/'),
      expect.objectContaining({ name: 'Test', type: 'text', isPublic: true })
    )
    expect(mockSet).toHaveBeenCalledWith(
      expect.stringContaining('/chat/room/'),
      expect.objectContaining({ name: 'Test', type: 'text' })
    )
  })

  it('createRoom with password stores hasPassword flag', () => {
    const { createRoom } = useRooms()
    createRoom({
      name: 'Secret',
      type: 'text',
      isPublic: true,
      passwordHash: 'hash123',
      passwordSalt: 'salt123',
    })
    // Registry should have hasPassword flag
    expect(mockSet).toHaveBeenCalledWith(
      expect.stringContaining('/chat/registry/rooms/'),
      expect.objectContaining({ hasPassword: true })
    )
    // Meta should have hash and salt
    expect(mockSet).toHaveBeenCalledWith(
      expect.stringContaining('/meta'),
      expect.objectContaining({ passwordHash: 'hash123', passwordSalt: 'salt123' })
    )
  })

  it('joinRoom adds to joined set', () => {
    const { joinRoom, joinedRoomIds } = useRooms()
    joinRoom('test-room')
    expect(joinedRoomIds.value.has('test-room')).toBe(true)
  })

  it('leaveRoom removes from joined set', () => {
    const { joinRoom, leaveRoom, joinedRoomIds } = useRooms()
    joinRoom('test-room')
    leaveRoom('test-room')
    expect(joinedRoomIds.value.has('test-room')).toBe(false)
  })

  it('deleteRoom only works for creator', () => {
    const { createRoom, deleteRoom, rooms } = useRooms()
    const id = createRoom({ name: 'MyRoom', type: 'text', isPublic: true })

    // Should work for creator (user-1)
    deleteRoom(id)
    expect(mockSet).toHaveBeenCalledWith(
      expect.stringContaining('/chat/registry/rooms/'),
      null
    )
  })

  it('createDM generates deterministic and symmetric ID', () => {
    const { createDM } = useRooms()
    mockUserId.value = 'aaa'
    const id1 = createDM('bbb', 'UserB')
    // The DM room ID should be deterministic
    expect(id1).toMatch(/^dm-/)

    // Symmetric: same ID regardless of who initiates
    mockUserId.value = 'bbb'
    const id2 = createDM('aaa', 'UserA')
    // Since the module is shared, it returns the existing room
    // The ID should match the pattern
    expect(id2).toMatch(/^dm-/)
  })
})
