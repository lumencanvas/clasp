<script setup>
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import EmojiPicker from './EmojiPicker.vue'
import { getRegisteredCommands } from '../lib/plugins.js'

const props = defineProps({
  replyTo: { type: Object, default: null },
  editingMessage: { type: Object, default: null },
})

const emit = defineEmits(['send', 'send-image', 'typing', 'cancel-reply', 'cancel-edit', 'save-edit'])

const message = ref('')
const inputRef = ref(null)
const fileInput = ref(null)
const showEmoji = ref(false)
const emojiRef = ref(null)
const selectedCmdIdx = ref(0)

const commandSuggestions = computed(() => {
  const text = message.value
  if (!text.startsWith('/') || text.includes(' ')) return []
  const partial = text.slice(1).toLowerCase()
  return getRegisteredCommands().filter(c => c.name.startsWith(partial))
})

// When editing starts, populate the input
watch(() => props.editingMessage, (msg) => {
  if (msg) {
    message.value = msg.text || ''
    inputRef.value?.focus()
  }
})

function handleSend() {
  if (props.editingMessage) {
    if (message.value.trim()) {
      emit('save-edit', props.editingMessage.msgId, message.value)
    }
    message.value = ''
    return
  }
  if (!message.value.trim()) return
  emit('send', message.value)
  message.value = ''
  inputRef.value?.focus()
}

function handleInput() {
  emit('typing')
  selectedCmdIdx.value = 0
}

function handleKeydown(e) {
  if (e.key === 'Escape') {
    if (props.editingMessage) emit('cancel-edit')
    else if (props.replyTo) emit('cancel-reply')
  }
  // Command autocomplete navigation
  if (commandSuggestions.value.length > 0) {
    if (e.key === 'Tab') {
      e.preventDefault()
      const cmd = commandSuggestions.value[selectedCmdIdx.value]
      if (cmd) {
        message.value = `/${cmd.name} `
      }
    } else if (e.key === 'ArrowDown') {
      e.preventDefault()
      selectedCmdIdx.value = Math.min(selectedCmdIdx.value + 1, commandSuggestions.value.length - 1)
    } else if (e.key === 'ArrowUp') {
      e.preventDefault()
      selectedCmdIdx.value = Math.max(selectedCmdIdx.value - 1, 0)
    }
  }
}

function selectCommand(cmd) {
  message.value = `/${cmd.name} `
  inputRef.value?.focus()
}

function openFilePicker() {
  fileInput.value?.click()
}

function handleFileChange(e) {
  const file = e.target.files?.[0]
  if (!file || !file.type.startsWith('image/')) return

  // Limit to ~500KB for base64 in message payload
  if (file.size > 512 * 1024) {
    return
  }

  const reader = new FileReader()
  reader.onload = () => {
    emit('send-image', reader.result)
  }
  reader.readAsDataURL(file)

  // Reset input so same file can be re-selected
  e.target.value = ''
}

function insertEmoji(emoji) {
  message.value += emoji
  showEmoji.value = false
  inputRef.value?.focus()
}

function handleClickOutside(e) {
  if (showEmoji.value && emojiRef.value && !emojiRef.value.contains(e.target)) {
    showEmoji.value = false
  }
}

onMounted(() => {
  inputRef.value?.focus()
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})

defineExpose({ focus: () => inputRef.value?.focus() })
</script>

<template>
  <div class="composer-wrapper">
    <!-- Reply preview bar -->
    <div v-if="replyTo" class="composer-context reply-bar">
      <div class="context-info">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="9 17 4 12 9 7"/>
          <path d="M20 18v-2a4 4 0 0 0-4-4H4"/>
        </svg>
        <span class="context-label">Replying to <strong>{{ replyTo.from }}</strong></span>
        <span class="context-text">{{ replyTo.text }}</span>
      </div>
      <button class="context-close" @click="emit('cancel-reply')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>

    <!-- Edit preview bar -->
    <div v-if="editingMessage" class="composer-context edit-bar">
      <div class="context-info">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
          <path d="M18.5 2.5a2.12 2.12 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
        </svg>
        <span class="context-label">Editing message</span>
      </div>
      <button class="context-close" @click="emit('cancel-edit')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>

    <!-- Command autocomplete -->
    <div v-if="commandSuggestions.length" class="command-suggestions">
      <button
        v-for="(cmd, i) in commandSuggestions"
        :key="cmd.name"
        :class="['cmd-suggestion', { active: i === selectedCmdIdx }]"
        @click="selectCommand(cmd)"
      >
        <span class="cmd-name">/{{ cmd.name }}</span>
        <span class="cmd-desc">{{ cmd.description }}</span>
      </button>
    </div>

    <div class="message-composer">
      <button class="attach-btn" title="Attach image" @click="openFilePicker">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
          <circle cx="8.5" cy="8.5" r="1.5"/>
          <polyline points="21 15 16 10 5 21"/>
        </svg>
      </button>
      <input type="file" ref="fileInput" accept="image/*" style="display:none" @change="handleFileChange" />

      <input
        ref="inputRef"
        v-model="message"
        type="text"
        :placeholder="editingMessage ? 'Edit message...' : replyTo ? 'Reply...' : 'Type a message...'"
        maxlength="2000"
        @keyup.enter="handleSend"
        @keydown="handleKeydown"
        @input="handleInput"
      />
      <div class="emoji-wrapper" ref="emojiRef">
        <button class="attach-btn" title="Emoji" @click.stop="showEmoji = !showEmoji">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <path d="M8 14s1.5 2 4 2 4-2 4-2"/>
            <line x1="9" y1="9" x2="9.01" y2="9"/>
            <line x1="15" y1="9" x2="15.01" y2="9"/>
          </svg>
        </button>
        <div v-if="showEmoji" class="emoji-popover">
          <EmojiPicker @select="insertEmoji" />
        </div>
      </div>
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
  </div>
</template>

<style scoped>
.composer-wrapper {
  flex-shrink: 0;
}

.composer-context {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  padding: 0.4rem 1rem;
  background: var(--bg-tertiary);
  border-top: 1px solid var(--border);
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.context-info {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  min-width: 0;
  overflow: hidden;
}

.context-info svg {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
}

.reply-bar .context-info svg { color: var(--accent); }
.edit-bar .context-info svg { color: var(--warning, #f59e0b); }

.context-label {
  flex-shrink: 0;
  white-space: nowrap;
}

.context-text {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  opacity: 0.7;
}

.context-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 3px;
  flex-shrink: 0;
}

.context-close svg {
  width: 14px;
  height: 14px;
}

.context-close:hover {
  background: var(--bg-active);
  color: var(--text-primary);
}

.message-composer {
  display: flex;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border);
}

.attach-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  background: transparent;
  border: none;
  border-radius: 50%;
  color: var(--text-muted);
  cursor: pointer;
  flex-shrink: 0;
  transition: color 0.15s;
}

.attach-btn svg {
  width: 18px;
  height: 18px;
}

.attach-btn:hover {
  color: var(--text-primary);
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

.command-suggestions {
  display: flex;
  flex-direction: column;
  background: var(--bg-tertiary);
  border-top: 1px solid var(--border);
  padding: 0.25rem;
}

.cmd-suggestion {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.4rem 0.75rem;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--text-secondary);
  font-size: 0.8rem;
  text-align: left;
  cursor: pointer;
  transition: background 0.1s;
}

.cmd-suggestion:hover,
.cmd-suggestion.active {
  background: var(--bg-active);
  color: var(--text-primary);
}

.cmd-name {
  font-weight: 700;
  color: var(--accent2);
  font-family: 'JetBrains Mono', monospace;
}

.cmd-desc {
  color: var(--text-muted);
}

.emoji-wrapper {
  position: relative;
}

.emoji-popover {
  position: absolute;
  bottom: 100%;
  right: 0;
  margin-bottom: 8px;
  z-index: 100;
}
</style>
