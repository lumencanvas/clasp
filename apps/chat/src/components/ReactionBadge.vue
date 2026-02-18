<script setup>
defineProps({
  emoji: { type: String, required: true },
  count: { type: Number, default: 0 },
  active: { type: Boolean, default: false },
})

const emit = defineEmits(['toggle'])
</script>

<template>
  <button
    :class="['reaction-badge', { active }]"
    @click="emit('toggle')"
  >
    <span class="reaction-emoji">{{ emoji }}</span>
    <span class="reaction-count">{{ count }}</span>
  </button>
</template>

<style scoped>
.reaction-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 12px;
  font-size: 0.75rem;
  cursor: pointer;
  transition: all 0.1s;
}

.reaction-badge:hover {
  background: var(--bg-active);
}

.reaction-badge.active {
  border-color: var(--accent);
  background: rgba(var(--accent-rgb), 0.1);
}

.reaction-emoji {
  font-size: 0.85rem;
  line-height: 1;
}

.reaction-count {
  color: var(--text-secondary);
  font-size: 0.7rem;
  font-weight: 600;
}
</style>
