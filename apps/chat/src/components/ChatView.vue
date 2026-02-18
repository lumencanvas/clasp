<script setup>
import { ref, toRef } from 'vue'
import { useChat } from '../composables/useChat.js'
import { useReactions } from '../composables/useReactions.js'
import { useCrypto } from '../composables/useCrypto.js'
import { useAdmin } from '../composables/useAdmin.js'
import MessageList from './MessageList.vue'
import MessageComposer from './MessageComposer.vue'
import TypingIndicator from './TypingIndicator.vue'
import AdminPanel from './AdminPanel.vue'

const props = defineProps({
  roomId: { type: String, required: true },
  isActive: { type: Boolean, default: true },
})

const roomIdRef = toRef(props, 'roomId')
const isActiveRef = toRef(props, 'isActive')

const {
  messages,
  sortedParticipants,
  typingList,
  onlineCount,
  replyTo,
  editingMessage,
  sendMessage,
  editMessage,
  deleteMessage,
  setReplyTo,
  startEditing,
  cancelEditing,
  handleTyping,
} = useChat(roomIdRef, isActiveRef)

const { toggleReaction, getMessageReactions } = useReactions(roomIdRef)
const { isEncrypted, enableEncryption } = useCrypto()
const { isRoomCreator, subscribeBans } = useAdmin(roomIdRef)

const showAdmin = ref(false)

// Subscribe to bans when room is active
subscribeBans()

const roomIsEncrypted = () => isEncrypted(props.roomId)

async function toggleEncryption() {
  if (!isEncrypted(props.roomId)) {
    await enableEncryption(props.roomId)
  }
}

function handleSend(text) {
  sendMessage(text)
}

function handleSendImage(dataUrl) {
  sendMessage('', { image: dataUrl })
}

defineExpose({ sortedParticipants, onlineCount })
</script>

<template>
  <div class="chat-view">
    <div class="chat-header-bar">
      <div v-if="roomIsEncrypted()" class="encryption-indicator">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="12" height="12">
        <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
        <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
      </svg>
      <span>End-to-end encrypted</span>
    </div>
      <button
        v-if="isRoomCreator"
        class="admin-toggle"
        aria-label="Room settings"
        title="Room settings"
        @click="showAdmin = !showAdmin"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
          <circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
        </svg>
      </button>
    </div>
    <AdminPanel
      v-if="showAdmin"
      :room-id="roomId"
      :members="sortedParticipants"
      @close="showAdmin = false"
    />
    <MessageList
      :messages="messages"
      :get-reactions="getMessageReactions"
      @reply="setReplyTo"
      @edit="startEditing"
      @delete="deleteMessage"
      @react="toggleReaction"
    />
    <TypingIndicator :users="typingList" />
    <MessageComposer
      :reply-to="replyTo"
      :editing-message="editingMessage"
      @send="handleSend"
      @send-image="handleSendImage"
      @typing="handleTyping"
      @cancel-reply="setReplyTo(null)"
      @cancel-edit="cancelEditing"
      @save-edit="editMessage"
    />
  </div>
</template>

<style scoped>
.chat-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

.chat-header-bar {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-shrink: 0;
}

.encryption-indicator {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.25rem 1rem;
  flex: 1;
  background: color-mix(in srgb, var(--success) 10%, transparent);
  border-bottom: 1px solid color-mix(in srgb, var(--success) 20%, transparent);
  font-size: 0.7rem;
  color: var(--accent3);
}

.admin-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--text-muted);
  cursor: pointer;
  flex-shrink: 0;
  margin-right: 0.5rem;
}

.admin-toggle:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}
</style>
