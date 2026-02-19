import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { ref, nextTick } from 'vue'

// Mock dependencies
const mockSet = vi.fn()
const mockEmit = vi.fn()
const mockSubscribe = vi.fn(() => vi.fn())
const mockConnected = ref(true)
const mockSessionId = ref('session-1')

vi.mock('../useClasp.js', () => ({
  useClasp: () => ({
    connected: mockConnected,
    sessionId: mockSessionId,
    set: mockSet,
    emit: mockEmit,
    subscribe: mockSubscribe,
  }),
}))

const mockUserId = ref('user-1')
const mockDisplayName = ref('TestUser')
const mockAvatarColor = ref('#e63946')

vi.mock('../useIdentity.js', () => ({
  useIdentity: () => ({
    userId: mockUserId,
    displayName: mockDisplayName,
    avatarColor: mockAvatarColor,
  }),
}))

vi.mock('../useNotifications.js', () => ({
  useNotifications: () => ({
    incrementUnread: vi.fn(),
    notifyMessage: vi.fn(),
  }),
}))

vi.mock('../useStorage.js', () => ({
  useStorage: () => ({
    loadCachedMessages: vi.fn(() => Promise.resolve([])),
    persistMessage: vi.fn(),
  }),
}))

// Mock crypto — controllable encrypt/decrypt/isEncrypted
const mockEncrypt = vi.fn(() => Promise.resolve(null))
const mockDecrypt = vi.fn(() => Promise.resolve(null))
const mockIsEncrypted = vi.fn(() => false)
const mockLoadRoomKey = vi.fn(() => Promise.resolve(null))
const mockSubscribeKeyExchange = vi.fn(() => vi.fn())
const mockEncryptedRooms = ref(new Set())
const mockMarkPasswordProtected = vi.fn()

vi.mock('../useCrypto.js', () => ({
  useCrypto: () => ({
    encrypt: mockEncrypt,
    decrypt: mockDecrypt,
    isEncrypted: mockIsEncrypted,
    loadRoomKey: mockLoadRoomKey,
    subscribeKeyExchange: mockSubscribeKeyExchange,
    encryptedRooms: mockEncryptedRooms,
    markPasswordProtected: mockMarkPasswordProtected,
  }),
}))

// Mock plugins
vi.mock('../../lib/plugins.js', () => ({
  executeCommand: vi.fn(() => false),
  initBuiltinPlugins: vi.fn(() => Promise.resolve()),
  getRegisteredCommands: vi.fn(() => []),
}))

// Stub onUnmounted since we're not in a component lifecycle
vi.mock('vue', async () => {
  const actual = await vi.importActual('vue')
  return {
    ...actual,
    onUnmounted: vi.fn(),
  }
})

import { useChat } from '../useChat.js'

// Helper to flush all pending microtasks
function flushPromises() {
  return new Promise(resolve => {
    // Use queueMicrotask to ensure all pending microtasks are flushed
    queueMicrotask(() => queueMicrotask(() => queueMicrotask(resolve)))
  })
}

describe('useChat', () => {
  let roomId

  beforeEach(() => {
    vi.clearAllMocks()
    mockConnected.value = true
    mockSessionId.value = 'session-1'
    mockUserId.value = 'user-1'
    mockDisplayName.value = 'TestUser'
    mockAvatarColor.value = '#e63946'
    mockEncrypt.mockImplementation(() => Promise.resolve(null))
    mockDecrypt.mockImplementation(() => Promise.resolve(null))
    mockIsEncrypted.mockImplementation(() => false)
    mockLoadRoomKey.mockImplementation(() => Promise.resolve(null))
    mockEncryptedRooms.value = new Set()
    roomId = ref(null)
  })

  async function createChat(rid = 'room-1') {
    roomId.value = rid
    const chat = useChat(roomId, ref(true))
    // joinChat is async (has await loadCachedMessages + await loadRoomKey)
    // Flush multiple microtask levels to let it complete
    await flushPromises()
    return chat
  }

  describe('sendMessage', () => {
    it('emits to correct CLASP path with message data', async () => {
      const { sendMessage } = await createChat()
      await sendMessage('hello world')

      expect(mockEmit).toHaveBeenCalledWith(
        '/chat/room/room-1/messages',
        expect.objectContaining({
          from: 'TestUser',
          fromId: 'session-1',
          text: 'hello world',
          type: 'text',
        })
      )
    })

    it('rejects empty text (no emit called)', async () => {
      const { sendMessage } = await createChat()
      mockEmit.mockClear()
      await sendMessage('   ')

      const messageEmits = mockEmit.mock.calls.filter(
        c => typeof c[0] === 'string' && c[0].includes('/messages')
      )
      expect(messageEmits).toHaveLength(0)
    })

    it('with waitingForKey=true adds system message, does not emit', async () => {
      const { sendMessage, messages, waitingForKey } = await createChat()
      waitingForKey.value = true
      mockEmit.mockClear()

      await sendMessage('secret')

      const messageEmits = mockEmit.mock.calls.filter(
        c => typeof c[0] === 'string' && c[0].includes('/messages')
      )
      expect(messageEmits).toHaveLength(0)
      expect(messages.value.some(m => m.type === 'system' && m.text.includes('Waiting for room key'))).toBe(true)
    })

    it('in encrypted room calls encrypt() and emits encrypted payload', async () => {
      // Create chat first (isEncrypted=false so joinChat won't set waitingForKey)
      const { sendMessage, waitingForKey } = await createChat()

      // Now set isEncrypted to true for the sendMessage call
      mockIsEncrypted.mockReturnValue(true)
      mockEncrypt.mockResolvedValue({ ciphertext: 'encrypted-text', iv: 'test-iv' })
      mockEmit.mockClear()

      await sendMessage('secret message')

      expect(mockEncrypt).toHaveBeenCalledWith('room-1', 'secret message')
      expect(mockEmit).toHaveBeenCalledWith(
        '/chat/room/room-1/messages',
        expect.objectContaining({
          text: 'encrypted-text',
          iv: 'test-iv',
          encrypted: true,
        })
      )
    })

    it('in encrypted room encrypts image data separately', async () => {
      const { sendMessage } = await createChat()

      // Set encryption mocks AFTER createChat to avoid waitingForKey
      mockIsEncrypted.mockReturnValue(true)
      mockEncrypt
        .mockResolvedValueOnce({ ciphertext: 'enc-text', iv: 'iv-text' })
        .mockResolvedValueOnce({ ciphertext: 'enc-img', iv: 'iv-img' })
      mockEmit.mockClear()

      await sendMessage('with image', { image: 'data:image/png;base64,abc' })

      expect(mockEncrypt).toHaveBeenCalledTimes(2)
      expect(mockEmit).toHaveBeenCalledWith(
        '/chat/room/room-1/messages',
        expect.objectContaining({
          encrypted: true,
          encryptedImage: 'enc-img',
          imageIv: 'iv-img',
        })
      )
    })

    it('rate limiting — 6th message within 1s is throttled', async () => {
      const { sendMessage, messages } = await createChat()

      // Send 5 messages (the limit)
      for (let i = 0; i < 5; i++) {
        await sendMessage(`msg ${i}`)
      }

      mockEmit.mockClear()
      await sendMessage('overflow')

      // The 6th should be throttled
      const messageEmits = mockEmit.mock.calls.filter(
        c => typeof c[0] === 'string' && c[0].includes('/messages')
      )
      expect(messageEmits).toHaveLength(0)
      expect(messages.value.some(m => m.type === 'system' && m.text.includes('Slow down'))).toBe(true)
    })
  })

  describe('editMessage', () => {
    it('emits edit payload to messages path', async () => {
      const { sendMessage, editMessage, messages } = await createChat()
      await sendMessage('original')

      const msg = messages.value.find(m => m.msgId)
      expect(msg).toBeTruthy()

      mockEmit.mockClear()
      await editMessage(msg.msgId, 'edited text')

      expect(mockEmit).toHaveBeenCalledWith(
        '/chat/room/room-1/messages',
        expect.objectContaining({
          type: 'edit',
          targetId: msg.msgId,
          text: 'edited text',
        })
      )
    })

    it('in encrypted room encrypts the new text', async () => {
      const { sendMessage, editMessage, messages } = await createChat()

      await sendMessage('original')
      const msg = messages.value.find(m => m.msgId)
      expect(msg).toBeTruthy()

      // Set encryption AFTER createChat and after sending first message
      mockIsEncrypted.mockReturnValue(true)
      mockEncrypt.mockResolvedValue({ ciphertext: 'enc-edit', iv: 'edit-iv' })
      mockEmit.mockClear()

      await editMessage(msg.msgId, 'new text')

      expect(mockEmit).toHaveBeenCalledWith(
        '/chat/room/room-1/messages',
        expect.objectContaining({
          type: 'edit',
          text: 'enc-edit',
          iv: 'edit-iv',
          encrypted: true,
        })
      )
    })
  })

  describe('deleteMessage', () => {
    it('emits delete payload and removes from local messages', async () => {
      const { sendMessage, deleteMessage, messages } = await createChat()
      await sendMessage('to delete')

      const msg = messages.value.find(m => m.msgId)
      expect(msg).toBeTruthy()
      const msgId = msg.msgId

      mockEmit.mockClear()
      deleteMessage(msgId)

      expect(mockEmit).toHaveBeenCalledWith(
        '/chat/room/room-1/messages',
        expect.objectContaining({
          type: 'delete',
          targetId: msgId,
        })
      )
      expect(messages.value.find(m => m.msgId === msgId)).toBeUndefined()
    })
  })

  describe('handleIncomingMessage (via subscribe callback)', () => {
    function getMessagesCallback() {
      const call = mockSubscribe.mock.calls.find(
        c => typeof c[0] === 'string' && c[0].includes('/messages')
      )
      return call ? call[1] : null
    }

    it('decrypts encrypted messages', async () => {
      mockDecrypt.mockResolvedValue('decrypted text')
      const { messages } = await createChat()

      const onMessage = getMessagesCallback()
      expect(onMessage).toBeTruthy()

      await onMessage({
        from: 'PeerUser',
        fromId: 'session-2',
        msgId: 'msg-enc-1',
        text: 'ciphertext',
        iv: 'iv-data',
        encrypted: true,
        timestamp: Date.now(),
      })

      expect(mockDecrypt).toHaveBeenCalledWith('room-1', 'ciphertext', 'iv-data')
      const incoming = messages.value.find(m => m.msgId === 'msg-enc-1')
      expect(incoming).toBeTruthy()
      expect(incoming.text).toBe('decrypted text')
    })

    it('decrypts encrypted images', async () => {
      mockDecrypt
        .mockResolvedValueOnce('decrypted text')
        .mockResolvedValueOnce('data:image/png;base64,decrypted')

      const { messages } = await createChat()
      const onMessage = getMessagesCallback()

      await onMessage({
        from: 'PeerUser',
        fromId: 'session-2',
        msgId: 'msg-img-1',
        text: 'enc-text',
        iv: 'iv-text',
        encrypted: true,
        encryptedImage: 'enc-img',
        imageIv: 'iv-img',
        timestamp: Date.now(),
      })

      expect(mockDecrypt).toHaveBeenCalledWith('room-1', 'enc-img', 'iv-img')
      const incoming = messages.value.find(m => m.msgId === 'msg-img-1')
      expect(incoming.image).toBe('data:image/png;base64,decrypted')
    })

    it('shows placeholder for undecryptable messages in encrypted room', async () => {
      mockDecrypt.mockResolvedValue(null)
      mockIsEncrypted.mockReturnValue(true)

      const { messages } = await createChat()
      const onMessage = getMessagesCallback()

      await onMessage({
        from: 'PeerUser',
        fromId: 'session-2',
        msgId: 'msg-fail-1',
        text: 'garbled',
        iv: 'bad-iv',
        encrypted: true,
        timestamp: Date.now(),
      })

      const incoming = messages.value.find(m => m.msgId === 'msg-fail-1')
      expect(incoming).toBeTruthy()
      expect(incoming.text).toContain('encrypted message')
    })

    it('handles edit type — updates existing message text', async () => {
      const { messages } = await createChat()
      const onMessage = getMessagesCallback()

      // First add a message
      await onMessage({
        from: 'PeerUser',
        fromId: 'session-2',
        msgId: 'msg-to-edit',
        text: 'original',
        timestamp: Date.now(),
      })

      // Then send an edit
      await onMessage({
        type: 'edit',
        fromId: 'session-2',
        targetId: 'msg-to-edit',
        text: 'edited',
        timestamp: Date.now(),
      })

      const edited = messages.value.find(m => m.msgId === 'msg-to-edit')
      expect(edited.text).toBe('edited')
      expect(edited.edited).toBe(true)
    })

    it('handles delete type — removes message', async () => {
      const { messages } = await createChat()
      const onMessage = getMessagesCallback()

      await onMessage({
        from: 'PeerUser',
        fromId: 'session-2',
        msgId: 'msg-to-del',
        text: 'delete me',
        timestamp: Date.now(),
      })

      expect(messages.value.find(m => m.msgId === 'msg-to-del')).toBeTruthy()

      await onMessage({
        type: 'delete',
        fromId: 'session-2',
        targetId: 'msg-to-del',
        timestamp: Date.now(),
      })

      expect(messages.value.find(m => m.msgId === 'msg-to-del')).toBeUndefined()
    })

    it('skips own messages (fromId === sessionId)', async () => {
      const { messages } = await createChat()
      const onMessage = getMessagesCallback()

      await onMessage({
        from: 'TestUser',
        fromId: 'session-1', // same as mockSessionId
        msgId: 'own-msg',
        text: 'my message',
        timestamp: Date.now(),
      })

      // Should not add another copy (own messages added optimistically via sendMessage)
      const ownMsgs = messages.value.filter(m => m.msgId === 'own-msg')
      expect(ownMsgs).toHaveLength(0)
    })
  })

  describe('joinChat', () => {
    it('subscribes to messages, presence, typing, crypto, and room meta', async () => {
      await createChat()

      const subPaths = mockSubscribe.mock.calls.map(c => c[0])

      expect(subPaths.some(p => p.includes('/messages'))).toBe(true)
      expect(subPaths.some(p => p.includes('/presence/*'))).toBe(true)
      expect(subPaths.some(p => p.includes('/typing/*'))).toBe(true)
      expect(subPaths.some(p => p.includes('/meta'))).toBe(true)
      expect(mockSubscribeKeyExchange).toHaveBeenCalledWith('room-1')
    })
  })
})
