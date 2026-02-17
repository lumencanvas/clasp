<script setup>
import { computed } from 'vue'
import { useClasp } from '../composables/useClasp.js'
import { formatTime } from '../lib/utils.js'
import UserAvatar from './UserAvatar.vue'

const props = defineProps({
  message: { type: Object, required: true },
  grouped: { type: Boolean, default: false },
})

const { sessionId } = useClasp()
const isOwn = computed(() => props.message.fromId === sessionId.value)
</script>

<template>
  <div :class="['message', { own: isOwn, grouped }]">
    <UserAvatar
      v-if="!grouped"
      :name="message.from"
      :color="message.avatarColor"
      :size="36"
    />
    <div class="message-content">
      <div v-if="!grouped" class="message-meta">
        <span class="sender-name">{{ message.from }}</span>
        <span class="message-time">{{ formatTime(message.timestamp) }}</span>
      </div>
      <div class="message-bubble">{{ message.text }}</div>
    </div>
  </div>
</template>

<style scoped>
.message {
  display: flex;
  gap: 0.75rem;
  padding: 0.15rem 0;
}

.message.grouped {
  padding-left: 48px;
}

.message-content {
  max-width: 80%;
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
  min-width: 0;
}

.message-meta {
  display: flex;
  align-items: baseline;
  gap: 0.5rem;
}

.sender-name {
  font-size: 0.8rem;
  font-weight: 700;
  color: var(--text-primary);
}

.message-time {
  font-size: 0.65rem;
  color: var(--text-muted);
}

.message-bubble {
  font-size: 0.9rem;
  line-height: 1.45;
  color: var(--text-primary);
  word-break: break-word;
}
</style>
