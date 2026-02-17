<script setup>
import { toRef } from 'vue'
import { useChat } from '../composables/useChat.js'
import MessageList from './MessageList.vue'
import MessageComposer from './MessageComposer.vue'
import TypingIndicator from './TypingIndicator.vue'

const props = defineProps({
  roomId: { type: String, required: true },
})

const roomIdRef = toRef(props, 'roomId')

const {
  messages,
  sortedParticipants,
  typingList,
  onlineCount,
  sendMessage,
  handleTyping,
} = useChat(roomIdRef)

defineExpose({ sortedParticipants, onlineCount })
</script>

<template>
  <div class="chat-view">
    <MessageList :messages="messages" />
    <TypingIndicator :users="typingList" />
    <MessageComposer
      @send="sendMessage"
      @typing="handleTyping"
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
</style>
