<script setup>
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'
import MessageItem from './MessageItem.vue'
import SystemMessage from './SystemMessage.vue'

const props = defineProps({
  messages: { type: Array, default: () => [] },
  getReactions: { type: Function, default: null },
  isAdmin: { type: Boolean, default: false },
})

const emit = defineEmits(['reply', 'edit', 'delete', 'react', 'mod-delete'])

const scrollContainer = ref(null)
const ITEM_HEIGHT = 60 // estimated average message height
const BUFFER = 20 // extra items above/below viewport

const scrollTop = ref(0)
const containerHeight = ref(600)

const totalItems = computed(() => props.messages.length)

const startIdx = computed(() => {
  const idx = Math.floor(scrollTop.value / ITEM_HEIGHT) - BUFFER
  return Math.max(0, idx)
})

const endIdx = computed(() => {
  const idx = Math.ceil((scrollTop.value + containerHeight.value) / ITEM_HEIGHT) + BUFFER
  return Math.min(totalItems.value, idx)
})

const visibleMessages = computed(() => {
  return props.messages.slice(startIdx.value, endIdx.value)
})

const topPadding = computed(() => startIdx.value * ITEM_HEIGHT)
const bottomPadding = computed(() => (totalItems.value - endIdx.value) * ITEM_HEIGHT)

// Use full rendering for small message counts
const useVirtual = computed(() => totalItems.value > 200)

function isGrouped(idx) {
  if (idx === 0) return false
  const prev = props.messages[idx - 1]
  const curr = props.messages[idx]
  if (prev.type === 'system' || curr.type === 'system') return false
  return prev.fromId === curr.fromId
}

function isGroupedVirtual(localIdx) {
  const globalIdx = startIdx.value + localIdx
  return isGrouped(globalIdx)
}

function onScroll() {
  if (scrollContainer.value) {
    scrollTop.value = scrollContainer.value.scrollTop
  }
}

function scrollToBottom() {
  nextTick(() => {
    if (scrollContainer.value) {
      scrollContainer.value.scrollTop = scrollContainer.value.scrollHeight
    }
  })
}

let resizeObserver = null

onMounted(() => {
  if (scrollContainer.value) {
    containerHeight.value = scrollContainer.value.clientHeight
    resizeObserver = new ResizeObserver(entries => {
      for (const entry of entries) {
        containerHeight.value = entry.contentRect.height
      }
    })
    resizeObserver.observe(scrollContainer.value)
  }
})

onUnmounted(() => {
  resizeObserver?.disconnect()
})

watch(() => props.messages.length, () => {
  scrollToBottom()
})

defineExpose({ scrollToBottom })
</script>

<template>
  <div class="message-list" ref="scrollContainer" @scroll="onScroll">
    <div v-if="!messages.length" class="empty-state">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
        <path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"/>
      </svg>
      <p>No messages yet</p>
      <span>Be the first to say something!</span>
    </div>

    <!-- Virtual scrolling mode for large lists -->
    <template v-if="useVirtual && messages.length">
      <div :style="{ height: topPadding + 'px' }"></div>
      <template v-for="(msg, localIdx) in visibleMessages" :key="msg.id">
        <SystemMessage
          v-if="msg.type === 'system'"
          :text="msg.text"
        />
        <MessageItem
          v-else
          :message="msg"
          :grouped="isGroupedVirtual(localIdx)"
          :reactions="getReactions ? getReactions(msg.msgId) : []"
          :is-admin="isAdmin"
          @reply="emit('reply', $event)"
          @edit="emit('edit', $event)"
          @delete="emit('delete', $event)"
          @react="(msgId, emoji) => emit('react', msgId, emoji)"
          @mod-delete="emit('mod-delete', $event)"
        />
      </template>
      <div :style="{ height: bottomPadding + 'px' }"></div>
    </template>

    <!-- Standard rendering for small lists -->
    <template v-else>
      <template v-for="(msg, idx) in messages" :key="msg.id">
        <SystemMessage
          v-if="msg.type === 'system'"
          :text="msg.text"
        />
        <MessageItem
          v-else
          :message="msg"
          :grouped="isGrouped(idx)"
          :reactions="getReactions ? getReactions(msg.msgId) : []"
          :is-admin="isAdmin"
          @reply="emit('reply', $event)"
          @edit="emit('edit', $event)"
          @delete="emit('delete', $event)"
          @react="(msgId, emoji) => emit('react', msgId, emoji)"
          @mod-delete="emit('mod-delete', $event)"
        />
      </template>
    </template>
  </div>
</template>

<style scoped>
.message-list {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  color: var(--text-muted);
}

.empty-state svg {
  width: 48px;
  height: 48px;
  margin-bottom: 1rem;
  opacity: 0.5;
}

.empty-state p {
  font-size: 0.9rem;
  margin-bottom: 0.25rem;
}

.empty-state span {
  font-size: 0.8rem;
  opacity: 0.7;
}
</style>
