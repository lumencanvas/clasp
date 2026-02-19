<script setup>
import { computed } from 'vue'

const props = defineProps({
  users: { type: Array, default: () => [] },
})

const text = computed(() => {
  if (props.users.length === 0) return ''
  if (props.users.length === 1) return `${props.users[0]} is typing...`
  if (props.users.length === 2) return `${props.users[0]} and ${props.users[1]} are typing...`
  const otherCount = props.users.length - 1
  return `${props.users[0]} and ${otherCount === 1 ? '1 other is' : `${otherCount} others are`} typing...`
})
</script>

<template>
  <div v-if="users.length" class="typing-indicator">
    <div class="typing-dots">
      <span></span><span></span><span></span>
    </div>
    <span class="typing-text">{{ text }}</span>
  </div>
</template>

<style scoped>
.typing-indicator {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.25rem 1rem 0.5rem;
  min-height: 24px;
}

.typing-dots {
  display: flex;
  gap: 3px;
}

.typing-dots span {
  width: 5px;
  height: 5px;
  background: var(--text-muted);
  border-radius: 50%;
  animation: typing 1.4s infinite;
}

.typing-dots span:nth-child(2) { animation-delay: 0.2s; }
.typing-dots span:nth-child(3) { animation-delay: 0.4s; }

@keyframes typing {
  0%, 60%, 100% { transform: translateY(0); }
  30% { transform: translateY(-4px); }
}

.typing-text {
  font-size: 0.75rem;
  color: var(--text-muted);
  font-style: italic;
}
</style>
