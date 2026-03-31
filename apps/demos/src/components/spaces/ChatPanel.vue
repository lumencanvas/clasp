<script setup>
import { ref, watch, nextTick } from 'vue'

const props = defineProps({ messages: Array, show: Boolean })
const emit = defineEmits(['send'])

const text = ref('')
const messagesEl = ref(null)

watch(() => props.messages.length, async () => {
  await nextTick()
  if (messagesEl.value) messagesEl.value.scrollTop = messagesEl.value.scrollHeight
})

function send() {
  const t = text.value.trim()
  if (!t) return
  emit('send', t)
  text.value = ''
}
</script>

<template>
  <div v-if="show" class="chat-panel">
    <div ref="messagesEl" class="chat-messages">
      <div v-for="(m, i) in messages" :key="i" class="chat-msg">
        <span class="author" :style="{ color: m.color || 'var(--accent)' }">{{ m.userName || m.name }}</span>
        <span class="text">{{ m.text }}</span>
      </div>
    </div>
    <div class="chat-input-row">
      <input v-model="text" class="input" maxlength="200" placeholder="Say something..." @keydown.enter="send" />
    </div>
  </div>
</template>

<style scoped>
.chat-panel { border-top: 1px solid var(--border); background: var(--bg); max-height: 190px; display: flex; flex-direction: column; }
.chat-messages { flex: 1; padding: 8px 16px; overflow-y: auto; display: flex; flex-direction: column; gap: 3px; }
.chat-msg { font-size: 12px; line-height: 1.4; }
.author { font-weight: 600; margin-right: 6px; }
.text { color: var(--text2); }
.chat-input-row { padding: 8px 16px; border-top: 1px solid var(--border); }
.input { width: 100%; background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 8px 12px; color: var(--text); font-size: 13px; outline: none; }
.input:focus { border-color: var(--accent); }
</style>
