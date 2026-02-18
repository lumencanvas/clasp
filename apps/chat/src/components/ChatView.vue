<script setup>
import { toRef } from 'vue'
import { useChat } from '../composables/useChat.js'
import { useReactions } from '../composables/useReactions.js'
import { useCrypto } from '../composables/useCrypto.js'
import MessageList from './MessageList.vue'
import MessageComposer from './MessageComposer.vue'
import TypingIndicator from './TypingIndicator.vue'

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
    <div v-if="roomIsEncrypted()" class="encryption-indicator">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="12" height="12">
        <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
        <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
      </svg>
      <span>End-to-end encrypted</span>
    </div>
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

.encryption-indicator {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.25rem 1rem;
  background: rgba(42, 157, 143, 0.1);
  border-bottom: 1px solid rgba(42, 157, 143, 0.2);
  font-size: 0.7rem;
  color: var(--accent3);
}
</style>
