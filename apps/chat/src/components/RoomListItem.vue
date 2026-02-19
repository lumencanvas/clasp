<script setup>
import { computed } from 'vue'
import { ROOM_TYPE_INFO } from '../lib/constants.js'

const props = defineProps({
  room: { type: Object, required: true },
  active: { type: Boolean, default: false },
  unread: { type: Number, default: 0 },
})

const emit = defineEmits(['select'])

const typeInfo = computed(() => ROOM_TYPE_INFO[props.room.type] || ROOM_TYPE_INFO.text)
</script>

<template>
  <button :class="['room-item', { active }]" @click="emit('select', room.id)">
    <span class="room-icon" v-if="typeInfo.icon === '#'">#</span>
    <svg v-else-if="typeInfo.icon === 'cam'" class="room-icon-svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <polygon points="23 7 16 12 23 17 23 7"/>
      <rect x="1" y="5" width="15" height="14" rx="2" ry="2"/>
    </svg>
    <svg v-else class="room-icon-svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
      <line x1="3" y1="9" x2="21" y2="9"/>
      <line x1="9" y1="21" x2="9" y2="9"/>
    </svg>
    <span class="room-name">{{ room.name }}</span>
    <span v-if="unread > 0" class="unread-badge">{{ unread > 99 ? '99+' : unread }}</span>
  </button>
</template>

<style scoped>
.room-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.5rem 0.75rem;
  background: none;
  border: none;
  color: var(--text-secondary);
  font-size: 0.85rem;
  text-align: left;
  border-radius: 4px;
  transition: all 0.1s;
  min-height: 48px;
}

.room-item:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.room-item.active {
  background: var(--bg-active);
  color: var(--text-primary);
}

.room-item:active {
  background: var(--bg-active);
}

.room-icon {
  font-size: 1.1rem;
  opacity: 0.5;
  font-weight: 700;
  flex-shrink: 0;
  width: 20px;
  text-align: center;
}

.room-icon-svg {
  width: 16px;
  height: 16px;
  opacity: 0.5;
  flex-shrink: 0;
}

.room-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.unread-badge {
  background: var(--accent);
  color: white;
  font-size: 0.75rem;
  font-weight: 700;
  padding: 0.15rem 0.45rem;
  border-radius: 10px;
  min-width: 20px;
  text-align: center;
  flex-shrink: 0;
}
</style>
