<script setup>
import { ref } from 'vue'
import EmojiPicker from './EmojiPicker.vue'

const QUICK_EMOJIS = ['👍', '❤️', '😂', '😮', '😢', '🎉', '🔥', '👀']

const emit = defineEmits(['select'])
const showFull = ref(false)

function handleFullSelect(emoji) {
  emit('select', emoji)
  showFull.value = false
}
</script>

<template>
  <div class="reaction-picker">
    <button
      v-for="emoji in QUICK_EMOJIS"
      :key="emoji"
      class="emoji-btn"
      @click="emit('select', emoji)"
    >
      {{ emoji }}
    </button>
    <button class="emoji-btn more-btn" title="More" @click.stop="showFull = !showFull">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16">
        <circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/>
      </svg>
    </button>
    <div v-if="showFull" class="full-picker-popover">
      <EmojiPicker @select="handleFullSelect" />
    </div>
  </div>
</template>

<style scoped>
.reaction-picker {
  display: flex;
  gap: 2px;
  padding: 4px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 6px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.2);
  position: relative;
}

.emoji-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: transparent;
  border: none;
  border-radius: 4px;
  font-size: 1.1rem;
  cursor: pointer;
  transition: background 0.1s;
}

.emoji-btn:hover {
  background: var(--bg-active);
}

.more-btn {
  color: var(--text-muted);
}

.more-btn:hover {
  color: var(--text-primary);
}

.full-picker-popover {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 4px;
  z-index: 20;
}
</style>
