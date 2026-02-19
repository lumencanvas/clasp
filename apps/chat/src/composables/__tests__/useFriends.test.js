import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ref } from 'vue'

// Mock dependencies
const mockSet = vi.fn()
const mockEmit = vi.fn()
const mockSubscribe = vi.fn(() => vi.fn())
const mockConnected = ref(true)

vi.mock('../useClasp.js', () => ({
  useClasp: () => ({
    connected: mockConnected,
    set: mockSet,
    emit: mockEmit,
    subscribe: mockSubscribe,
  }),
}))

const mockUserId = ref('user-A')
const mockDisplayName = ref('Alice')
const mockAvatarColor = ref('#e63946')

vi.mock('../useIdentity.js', () => ({
  useIdentity: () => ({
    userId: mockUserId,
    displayName: mockDisplayName,
    avatarColor: mockAvatarColor,
  }),
}))

import { useFriends } from '../useFriends.js'

describe('useFriends', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockConnected.value = true
    mockUserId.value = 'user-A'
    mockDisplayName.value = 'Alice'
    mockAvatarColor.value = '#e63946'

    // Reset module-level state
    const { friends, pendingRequests, cleanup } = useFriends()
    cleanup()
    friends.value = new Map()
    pendingRequests.value = []
  })

  describe('init', () => {
    it('subscribes to friends list and incoming requests', () => {
      const { init } = useFriends()
      init()

      const subPaths = mockSubscribe.mock.calls.map(c => c[0])
      expect(subPaths.some(p => p.includes('/chat/user/user-A/friends/*'))).toBe(true)
      expect(subPaths.some(p => p.includes('/chat/requests/user-A'))).toBe(true)
    })
  })

  describe('sendRequest', () => {
    it('emits to /chat/requests/{targetId} with fromId, fromName, fromColor', () => {
      const { sendRequest } = useFriends()
      sendRequest('user-B', 'Bob')

      expect(mockEmit).toHaveBeenCalledWith(
        '/chat/requests/user-B',
        expect.objectContaining({
          fromId: 'user-A',
          fromName: 'Alice',
          fromColor: '#e63946',
          timestamp: expect.any(Number),
        })
      )
    })

    it('blocks self-friending (targetId === userId)', () => {
      const { sendRequest } = useFriends()
      sendRequest('user-A', 'Alice')
      expect(mockEmit).not.toHaveBeenCalled()
    })

    it('blocks duplicate friend request (already friends)', () => {
      const { sendRequest, friends } = useFriends()
      friends.value.set('user-B', { name: 'Bob', addedAt: Date.now() })

      sendRequest('user-B', 'Bob')
      expect(mockEmit).not.toHaveBeenCalled()
    })

    it('does nothing when disconnected', () => {
      mockConnected.value = false
      const { sendRequest } = useFriends()
      sendRequest('user-B', 'Bob')
      expect(mockEmit).not.toHaveBeenCalled()
    })
  })

  describe('acceptRequest', () => {
    it('writes own side via set() at friends path', () => {
      const { acceptRequest, pendingRequests } = useFriends()
      pendingRequests.value = [
        { fromId: 'user-B', fromName: 'Bob', fromColor: '#457b9d', timestamp: 1000 },
      ]

      acceptRequest('user-B')

      expect(mockSet).toHaveBeenCalledWith(
        '/chat/user/user-A/friends/user-B',
        expect.objectContaining({
          name: 'Bob',
          avatarColor: '#457b9d',
          addedAt: expect.any(Number),
        })
      )
    })

    it('emits confirmation to requester via emit() at requests path', () => {
      const { acceptRequest, pendingRequests } = useFriends()
      pendingRequests.value = [
        { fromId: 'user-B', fromName: 'Bob', fromColor: '#457b9d', timestamp: 1000 },
      ]

      acceptRequest('user-B')

      expect(mockEmit).toHaveBeenCalledWith(
        '/chat/requests/user-B',
        expect.objectContaining({
          type: 'accepted',
          fromId: 'user-A',
          fromName: 'Alice',
          fromColor: '#e63946',
        })
      )
    })

    it('removes request from pendingRequests', () => {
      const { acceptRequest, pendingRequests } = useFriends()
      pendingRequests.value = [
        { fromId: 'user-B', fromName: 'Bob', fromColor: '#457b9d', timestamp: 1000 },
      ]

      acceptRequest('user-B')
      expect(pendingRequests.value.find(r => r.fromId === 'user-B')).toBeUndefined()
    })
  })

  describe('incoming request handling', () => {
    it('incoming request adds to pendingRequests', () => {
      const { init, pendingRequests } = useFriends()
      init()

      // Find the requests subscription callback
      const reqSubCall = mockSubscribe.mock.calls.find(
        c => typeof c[0] === 'string' && c[0].includes('/chat/requests/user-A')
      )
      expect(reqSubCall).toBeTruthy()
      const onRequest = reqSubCall[1]

      onRequest({
        fromId: 'user-C',
        fromName: 'Charlie',
        fromColor: '#2a9d8f',
        timestamp: Date.now(),
      })

      expect(pendingRequests.value.some(r => r.fromId === 'user-C')).toBe(true)
    })

    it('incoming duplicate request is deduplicated (same fromId)', () => {
      const { init, pendingRequests } = useFriends()
      init()

      const reqSubCall = mockSubscribe.mock.calls.find(
        c => typeof c[0] === 'string' && c[0].includes('/chat/requests/user-A')
      )
      const onRequest = reqSubCall[1]

      onRequest({ fromId: 'user-D', fromName: 'Dave', fromColor: '#f77f00', timestamp: 1 })
      onRequest({ fromId: 'user-D', fromName: 'Dave', fromColor: '#f77f00', timestamp: 2 })

      const fromD = pendingRequests.value.filter(r => r.fromId === 'user-D')
      expect(fromD).toHaveLength(1)
    })

    it('incoming type:accepted confirmation writes friend record via set()', () => {
      const { init } = useFriends()
      init()

      const reqSubCall = mockSubscribe.mock.calls.find(
        c => typeof c[0] === 'string' && c[0].includes('/chat/requests/user-A')
      )
      const onRequest = reqSubCall[1]

      onRequest({
        type: 'accepted',
        fromId: 'user-E',
        fromName: 'Eve',
        fromColor: '#9b5de5',
        timestamp: Date.now(),
      })

      expect(mockSet).toHaveBeenCalledWith(
        '/chat/user/user-A/friends/user-E',
        expect.objectContaining({
          name: 'Eve',
          avatarColor: '#9b5de5',
          addedAt: expect.any(Number),
        })
      )
    })
  })

  describe('declineRequest', () => {
    it('removes from pendingRequests without set/emit', () => {
      const { declineRequest, pendingRequests } = useFriends()
      pendingRequests.value = [
        { fromId: 'user-F', fromName: 'Frank', fromColor: '#00bbf9', timestamp: 1000 },
      ]

      declineRequest('user-F')

      expect(pendingRequests.value.find(r => r.fromId === 'user-F')).toBeUndefined()
      expect(mockSet).not.toHaveBeenCalled()
      expect(mockEmit).not.toHaveBeenCalled()
    })
  })

  describe('removeFriend', () => {
    it('calls set(null) on own friends path', () => {
      const { removeFriend } = useFriends()
      removeFriend('user-G')

      expect(mockSet).toHaveBeenCalledWith(
        '/chat/user/user-A/friends/user-G',
        null
      )
    })
  })

  describe('isFriend', () => {
    it('returns true/false correctly', () => {
      const { isFriend, friends } = useFriends()

      expect(isFriend('user-H')).toBe(false)
      friends.value.set('user-H', { name: 'Hank', addedAt: Date.now() })
      expect(isFriend('user-H')).toBe(true)
    })
  })

  describe('friendList', () => {
    it('computed sorts alphabetically by name', () => {
      const { friendList, friends } = useFriends()

      friends.value = new Map([
        ['z-id', { name: 'Zara', addedAt: 1 }],
        ['a-id', { name: 'Alice', addedAt: 2 }],
        ['m-id', { name: 'Mike', addedAt: 3 }],
      ])

      expect(friendList.value.map(f => f.name)).toEqual(['Alice', 'Mike', 'Zara'])
    })
  })

  describe('cleanup', () => {
    it('unsubscribes and resets state', () => {
      const unsubA = vi.fn()
      const unsubB = vi.fn()
      mockSubscribe.mockReturnValueOnce(unsubA).mockReturnValueOnce(unsubB)

      const { init, cleanup } = useFriends()
      init()

      cleanup()
      expect(unsubA).toHaveBeenCalled()
      expect(unsubB).toHaveBeenCalled()
    })
  })
})
