<script setup>
import { ref, computed } from 'vue'
import { useClasp } from '../composables/useClasp.js'
import { formatTime } from '../lib/utils.js'
import { renderMarkdown } from '../lib/markdown.js'
import UserAvatar from './UserAvatar.vue'
import ReactionBadge from './ReactionBadge.vue'
import ReactionPicker from './ReactionPicker.vue'

const props = defineProps({
  message: { type: Object, required: true },
  grouped: { type: Boolean, default: false },
  reactions: { type: Array, default: () => [] },
  isAdmin: { type: Boolean, default: false },
})

const emit = defineEmits(['reply', 'edit', 'delete', 'react', 'mod-delete'])

const { sessionId } = useClasp()
const isOwn = computed(() => props.message.fromId === sessionId.value)
const showActions = ref(false)
const showPicker = ref(false)
let longPressTimer = null

function onTouchStart() {
  longPressTimer = setTimeout(() => {
    showActions.value = true
  }, 500)
}

function onTouchEnd() {
  clearTimeout(longPressTimer)
}

const renderedText = computed(() => renderMarkdown(props.message.text))

function handleReact(emoji) {
  emit('react', props.message.msgId, emoji)
  showPicker.value = false
}
</script>

<template>
  <div
    :class="['message', { own: isOwn, grouped }]"
    @mouseenter="showActions = true"
    @mouseleave="showActions = false; showPicker = false"
    @touchstart.passive="onTouchStart"
    @touchend.passive="onTouchEnd"
    @touchcancel.passive="onTouchEnd"
  >
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
        <span v-if="message.verified === 'verified'" class="verified-icon" title="Signature verified">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" width="11" height="11">
            <polyline points="20 6 9 17 4 12"/>
          </svg>
        </span>
        <span v-else-if="message.verified === 'failed'" class="verify-failed-icon" title="Signature verification failed">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" width="11" height="11">
            <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </span>
        <span v-if="message.edited" class="edited-tag">(edited)</span>
      </div>

      <!-- Reply preview -->
      <div v-if="message.replyTo" class="reply-preview">
        <span class="reply-author">{{ message.replyTo.from }}</span>
        <span class="reply-text">{{ message.replyTo.text }}</span>
      </div>

      <!-- Message text with markdown -->
      <div v-if="message.text" class="message-bubble" v-html="renderedText"></div>

      <!-- Inline image -->
      <img
        v-if="message.image"
        :src="message.image"
        class="message-image"
        loading="lazy"
      />

      <!-- Reactions -->
      <div v-if="reactions.length" class="reactions-row">
        <ReactionBadge
          v-for="r in reactions"
          :key="r.emoji"
          :emoji="r.emoji"
          :count="r.count"
          :active="r.active"
          @toggle="emit('react', message.msgId, r.emoji)"
        />
      </div>

      <!-- Action buttons (hover) -->
      <div v-if="showActions" class="message-actions">
        <button class="action-btn" title="React" aria-label="React" @click.stop="showPicker = !showPicker">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <path d="M8 14s1.5 2 4 2 4-2 4-2"/>
            <line x1="9" y1="9" x2="9.01" y2="9"/>
            <line x1="15" y1="9" x2="15.01" y2="9"/>
          </svg>
        </button>
        <button class="action-btn" title="Reply" aria-label="Reply" @click="emit('reply', message)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="9 17 4 12 9 7"/>
            <path d="M20 18v-2a4 4 0 0 0-4-4H4"/>
          </svg>
        </button>
        <template v-if="isOwn">
          <button class="action-btn" title="Edit" aria-label="Edit" @click="emit('edit', message)">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
              <path d="M18.5 2.5a2.12 2.12 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
            </svg>
          </button>
          <button class="action-btn action-danger" title="Delete" aria-label="Delete" @click="emit('delete', message.msgId)">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="3 6 5 6 21 6"/>
              <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
            </svg>
          </button>
        </template>
        <button v-if="isAdmin && !isOwn" class="action-btn action-danger" title="Mod Delete" aria-label="Mod Delete" @click="emit('mod-delete', message.msgId)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
          </svg>
        </button>

        <!-- Reaction picker popover -->
        <div v-if="showPicker" class="picker-popover">
          <ReactionPicker @select="handleReact" />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.message {
  display: flex;
  gap: 0.75rem;
  padding: 0.15rem 0;
  position: relative;
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
  font-size: 0.75rem;
  color: var(--text-muted);
}

.verified-icon {
  display: inline-flex;
  align-items: center;
  color: var(--accent3);
  opacity: 0.7;
}

.verify-failed-icon {
  display: inline-flex;
  align-items: center;
  color: var(--danger);
}

.edited-tag {
  font-size: 0.75rem;
  color: var(--text-muted);
  font-style: italic;
}

.reply-preview {
  display: flex;
  gap: 0.5rem;
  padding: 0.25rem 0.5rem;
  border-left: 2px solid var(--accent);
  background: var(--bg-tertiary);
  border-radius: 0 4px 4px 0;
  font-size: 0.75rem;
  color: var(--text-secondary);
  overflow: hidden;
}

.reply-author {
  font-weight: 600;
  flex-shrink: 0;
}

.reply-text {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.message-bubble {
  font-size: 0.9rem;
  line-height: 1.45;
  color: var(--text-primary);
  word-break: break-word;
}

.message-bubble :deep(strong) {
  font-weight: 700;
}

.message-bubble :deep(em) {
  font-style: italic;
}

.message-bubble :deep(del) {
  text-decoration: line-through;
  opacity: 0.7;
}

.message-bubble :deep(code) {
  padding: 0.15em 0.35em;
  background: var(--bg-tertiary);
  border-radius: 3px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.85em;
}

.message-bubble :deep(pre) {
  margin: 0.25rem 0;
  padding: 0.5rem 0.75rem;
  background: var(--bg-tertiary);
  border-radius: 4px;
  overflow-x: auto;
}

.message-bubble :deep(pre code) {
  padding: 0;
  background: transparent;
  font-size: 0.8rem;
  line-height: 1.5;
}

.message-image {
  max-width: min(320px, 100%);
  max-height: 240px;
  height: auto;
  border-radius: 4px;
  margin-top: 0.25rem;
  cursor: pointer;
  object-fit: contain;
  background: var(--bg-tertiary);
}

.reactions-row {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-top: 0.2rem;
}

.message-actions {
  position: absolute;
  top: -4px;
  right: 0;
  display: flex;
  gap: 2px;
  padding: 2px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 4px;
  box-shadow: 0 1px 4px rgba(0,0,0,0.15);
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background: transparent;
  border: none;
  border-radius: 3px;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.1s;
}

@media (hover: hover) and (pointer: fine) {
  .action-btn {
    width: 28px;
    height: 28px;
  }
}

.action-btn svg {
  width: 14px;
  height: 14px;
}

.action-btn:hover {
  background: var(--bg-active);
  color: var(--text-primary);
}

.action-btn:active {
  transform: scale(0.96);
  opacity: 0.8;
}

.action-btn.action-danger:hover {
  color: var(--danger);
}

.picker-popover {
  position: fixed;
  z-index: var(--z-popover);
  max-height: 50vh;
  overflow-y: auto;
}
</style>
