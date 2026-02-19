import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ref, nextTick } from 'vue'

// Mock dependencies
const mockSet = vi.fn()
const mockEmit = vi.fn()
const mockSubscribe = vi.fn(() => vi.fn())
const mockConnected = ref(true)
const mockUserId = ref('user-1')
const mockCurrentRoom = ref({ creatorId: 'user-1', name: 'test-room' })

vi.mock('../useClasp.js', () => ({
  useClasp: () => ({
    connected: mockConnected,
    set: mockSet,
    emit: mockEmit,
    subscribe: mockSubscribe,
  }),
}))

vi.mock('../useIdentity.js', () => ({
  useIdentity: () => ({
    userId: mockUserId,
  }),
}))

vi.mock('../useRooms.js', () => ({
  useRooms: () => ({
    currentRoom: mockCurrentRoom,
  }),
}))

vi.mock('../useCrypto.js', () => ({
  useCrypto: () => ({
    isEncrypted: vi.fn(() => false),
    rotateRoomKey: vi.fn(),
    removePeerKey: vi.fn(),
  }),
}))

import { useAdmin } from '../useAdmin.js'

describe('useAdmin', () => {
  const roomId = ref('room-1')

  beforeEach(() => {
    vi.clearAllMocks()
    mockConnected.value = true
    mockUserId.value = 'user-1'
    mockCurrentRoom.value = { creatorId: 'user-1', name: 'test-room' }
  })

  it('isRoomCreator returns true when userId matches creatorId', () => {
    const { isRoomCreator } = useAdmin(roomId)
    expect(isRoomCreator.value).toBe(true)
  })

  it('isRoomCreator returns false for non-creator', () => {
    mockUserId.value = 'user-2'
    const { isRoomCreator } = useAdmin(roomId)
    expect(isRoomCreator.value).toBe(false)
  })

  it('isAdmin returns true for room creator', () => {
    const { isAdmin } = useAdmin(roomId)
    expect(isAdmin.value).toBe(true)
  })

  it('isAdmin returns false for non-creator non-admin', () => {
    mockUserId.value = 'user-2'
    const { isAdmin } = useAdmin(roomId)
    expect(isAdmin.value).toBe(false)
  })

  it('kickUser calls set to clear presence', () => {
    const { kickUser } = useAdmin(roomId)
    kickUser('user-2')
    expect(mockSet).toHaveBeenCalledWith('/chat/room/room-1/presence/user-2', null)
  })

  it('banUser sets ban record and kicks', () => {
    const { banUser } = useAdmin(roomId)
    banUser('user-2')

    // Should set ban record
    expect(mockSet).toHaveBeenCalledWith(
      '/chat/room/room-1/bans/user-2',
      expect.objectContaining({ bannedBy: 'user-1' })
    )
    // Should also clear presence (kick)
    expect(mockSet).toHaveBeenCalledWith('/chat/room/room-1/presence/user-2', null)
  })

  it('subscribeBans returns unsubscribe function', () => {
    const { subscribeBans } = useAdmin(roomId)
    const unsub = subscribeBans()
    expect(mockSubscribe).toHaveBeenCalledWith(
      '/chat/room/room-1/bans/*',
      expect.any(Function)
    )
  })

  it('promoteToAdmin only works for creator', () => {
    mockUserId.value = 'user-2'
    const { promoteToAdmin } = useAdmin(roomId)
    promoteToAdmin('user-3')
    expect(mockSet).not.toHaveBeenCalled()
  })

  it('promoteToAdmin sets admin record for creator', () => {
    const { promoteToAdmin } = useAdmin(roomId)
    promoteToAdmin('user-2')
    expect(mockSet).toHaveBeenCalledWith(
      '/chat/room/room-1/admin/user-2',
      expect.objectContaining({ role: 'admin', promotedBy: 'user-1' })
    )
  })
})
