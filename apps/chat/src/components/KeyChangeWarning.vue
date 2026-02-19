<script setup>
import { computed } from 'vue'
import { useKeyVerification } from '../composables/useKeyVerification.js'

const props = defineProps({
  roomId: { type: String, required: true },
})

const { getRoomWarnings, acceptKeyChange, dismissWarning } = useKeyVerification()

const warnings = computed(() => getRoomWarnings(props.roomId))
</script>

<template>
  <div v-for="w in warnings" :key="w.userId" class="key-warning">
    <div class="warning-icon">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16">
        <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
        <line x1="12" y1="9" x2="12" y2="13"/>
        <line x1="12" y1="17" x2="12.01" y2="17"/>
      </svg>
    </div>
    <div class="warning-text">
      <strong>{{ w.displayName }}'s encryption key has changed.</strong>
      This could indicate a new device or a security compromise.
    </div>
    <div class="warning-actions">
      <button class="accept-btn" @click="acceptKeyChange(w.roomId, w.userId)">Accept</button>
      <button class="dismiss-btn" @click="dismissWarning(w.roomId, w.userId)">Dismiss</button>
    </div>
  </div>
</template>

<style scoped>
.key-warning {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  background: color-mix(in srgb, var(--danger) 12%, transparent);
  border-bottom: 1px solid color-mix(in srgb, var(--danger) 25%, transparent);
  flex-shrink: 0;
}

.warning-icon {
  color: var(--danger);
  flex-shrink: 0;
  margin-top: 1px;
}

.warning-text {
  flex: 1;
  font-size: 0.75rem;
  color: var(--text-secondary);
  line-height: 1.4;
}

.warning-text strong {
  color: var(--text-primary);
}

.warning-actions {
  display: flex;
  gap: 0.35rem;
  flex-shrink: 0;
}

.accept-btn,
.dismiss-btn {
  padding: 0.4rem 0.65rem;
  min-height: 36px;
  border: none;
  border-radius: 3px;
  font-size: 0.75rem;
  cursor: pointer;
  display: flex;
  align-items: center;
}

.accept-btn {
  background: var(--accent);
  color: white;
}

.accept-btn:hover {
  filter: brightness(1.1);
}

.dismiss-btn {
  background: var(--bg-tertiary);
  color: var(--text-muted);
}

.dismiss-btn:hover {
  color: var(--text-primary);
}
</style>
