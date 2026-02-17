<script setup>
import { ref, onMounted } from 'vue'

const emit = defineEmits(['send', 'typing'])

const message = ref('')
const inputRef = ref(null)

function handleSend() {
  if (!message.value.trim()) return
  emit('send', message.value)
  message.value = ''
  inputRef.value?.focus()
}

function handleInput() {
  emit('typing')
}

onMounted(() => {
  inputRef.value?.focus()
})

defineExpose({ focus: () => inputRef.value?.focus() })
</script>

<template>
  <div class="message-composer">
    <input
      ref="inputRef"
      v-model="message"
      type="text"
      placeholder="Type a message..."
      maxlength="2000"
      @keyup.enter="handleSend"
      @input="handleInput"
    />
    <button
      class="send-btn"
      @click="handleSend"
      :disabled="!message.trim()"
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="22" y1="2" x2="11" y2="13"/>
        <polygon points="22 2 15 22 11 13 2 9 22 2"/>
      </svg>
    </button>
  </div>
</template>

<style scoped>
.message-composer {
  display: flex;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}

.message-composer input {
  flex: 1;
  padding: 0.7rem 1rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 20px;
  font-size: 0.9rem;
  transition: border-color 0.15s;
}

.message-composer input:focus {
  outline: none;
  border-color: var(--accent);
}

.send-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  background: var(--accent);
  border: none;
  border-radius: 50%;
  color: white;
  transition: opacity 0.15s;
  flex-shrink: 0;
}

.send-btn svg {
  width: 16px;
  height: 16px;
}

.send-btn:hover:not(:disabled) {
  opacity: 0.9;
}

.send-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}
</style>
