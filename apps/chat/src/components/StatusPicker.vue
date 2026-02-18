<script setup>
import { USER_STATUSES } from '../lib/constants.js'

const props = defineProps({
  currentStatus: { type: String, default: 'online' },
})

const emit = defineEmits(['select', 'close'])
</script>

<template>
  <div class="popup-overlay" @click.self="emit('close')">
    <div class="status-picker">
      <div class="picker-header">Set Status</div>
      <button
        v-for="s in USER_STATUSES"
        :key="s.value"
        :class="['status-option', { active: currentStatus === s.value }]"
        @click="emit('select', s.value)"
      >
        <span class="status-dot" :style="{ background: s.color }"></span>
        <span class="status-label">{{ s.label }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.popup-overlay {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: flex-end;
  justify-content: flex-start;
  background: rgba(0,0,0,0.3);
  z-index: 200;
  padding: 0 0 4rem 0.75rem;
}

.status-picker {
  width: 200px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 0.5rem;
  box-shadow: 0 4px 16px rgba(0,0,0,0.3);
}

.picker-header {
  padding: 0.35rem 0.5rem;
  font-size: 0.65rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--text-muted);
  font-weight: 700;
}

.status-option {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.5rem;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--text-secondary);
  font-size: 0.8rem;
  text-align: left;
  cursor: pointer;
  transition: all 0.1s;
}

.status-option:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.status-option.active {
  background: var(--bg-active);
  color: var(--text-primary);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-label {
  flex: 1;
}
</style>
