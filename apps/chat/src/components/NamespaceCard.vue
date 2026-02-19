<script setup>
import { computed } from 'vue'

const props = defineProps({
  namespace: { type: Object, required: true },
  isPinned: { type: Boolean, default: false },
  roomCount: { type: Number, default: 0 },
})

const emit = defineEmits(['click', 'pin', 'unpin'])

const displayName = computed(() => props.namespace.path.split('/').pop())
const hasParent = computed(() => props.namespace.path.includes('/'))
const parentPath = computed(() => {
  const parts = props.namespace.path.split('/')
  return parts.slice(0, -1).join('/')
})
</script>

<template>
  <div class="ns-card" @click="emit('click', namespace.path)">
    <div class="card-top">
      <span v-if="namespace.icon" class="card-icon">{{ namespace.icon }}</span>
      <span v-else class="card-icon-default">/</span>
      <span class="card-count">
        <template v-if="roomCount > 0">{{ roomCount }} room{{ roomCount !== 1 ? 's' : '' }}</template>
        <template v-else>Browse
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="12" height="12"><polyline points="9 18 15 12 9 6"/></svg>
        </template>
      </span>
    </div>
    <span v-if="hasParent" class="card-parent">{{ parentPath }}/</span>
    <h4 class="card-name">{{ displayName }}</h4>
    <p v-if="namespace.description" class="card-desc">{{ namespace.description }}</p>
    <p v-if="namespace.creatorName" class="card-creator">by {{ namespace.creatorName }}</p>
    <button
      :class="['pin-btn', { pinned: isPinned }]"
      @click.stop="isPinned ? emit('unpin', namespace.path) : emit('pin', namespace.path)"
    >
      {{ isPinned ? 'Pinned' : 'Pin' }}
    </button>
  </div>
</template>

<style scoped>
.ns-card {
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  cursor: pointer;
  transition: border-color 0.15s;
}

.ns-card:hover {
  border-color: var(--text-muted);
}

.ns-card:active {
  transform: scale(0.98);
  opacity: 0.9;
}

.card-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-icon {
  font-size: 1.1rem;
}

.card-icon-default {
  font-size: 1.1rem;
  font-weight: 700;
  color: var(--accent2);
  opacity: 0.7;
}

.card-count {
  font-size: 0.75rem;
  color: var(--text-muted);
  display: inline-flex;
  align-items: center;
  gap: 0.2rem;
}

.card-parent {
  font-size: 0.75rem;
  color: var(--text-muted);
  font-family: var(--font-code);
  margin-bottom: -0.2rem;
}

.card-name {
  font-size: 0.95rem;
  letter-spacing: 0.04em;
}

.card-desc {
  font-size: 0.75rem;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.card-creator {
  font-size: 0.7rem;
  color: var(--text-muted);
}

.pin-btn {
  margin-top: 0.4rem;
  padding: 0.45rem;
  background: var(--accent);
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 0.75rem;
  transition: opacity 0.15s;
}

.pin-btn:hover:not(.pinned) {
  opacity: 0.9;
}

.pin-btn.pinned {
  background: var(--bg-active);
  color: var(--text-muted);
  cursor: default;
}
</style>
