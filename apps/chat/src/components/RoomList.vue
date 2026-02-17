<script setup>
import { computed } from 'vue'
import { ROOM_TYPES } from '../lib/constants.js'
import RoomListItem from './RoomListItem.vue'

const props = defineProps({
  rooms: { type: Array, default: () => [] },
  currentRoomId: { type: String, default: null },
  unreadCounts: { type: Map, default: () => new Map() },
})

const emit = defineEmits(['select'])

const textRooms = computed(() => props.rooms.filter(r => r.type === ROOM_TYPES.TEXT))
const videoRooms = computed(() => props.rooms.filter(r => r.type === ROOM_TYPES.VIDEO))
const comboRooms = computed(() => props.rooms.filter(r => r.type === ROOM_TYPES.COMBO))

const sections = computed(() => {
  const s = []
  if (textRooms.value.length) s.push({ label: 'Text Channels', rooms: textRooms.value })
  if (videoRooms.value.length) s.push({ label: 'Video Channels', rooms: videoRooms.value })
  if (comboRooms.value.length) s.push({ label: 'Combo Channels', rooms: comboRooms.value })
  return s
})
</script>

<template>
  <div class="room-list">
    <div v-if="!rooms.length" class="empty">
      <p>No rooms yet</p>
      <span>Create one to get started</span>
    </div>
    <div v-for="section in sections" :key="section.label" class="room-section">
      <div class="section-label">{{ section.label }}</div>
      <RoomListItem
        v-for="room in section.rooms"
        :key="room.id"
        :room="room"
        :active="room.id === currentRoomId"
        :unread="unreadCounts.get(room.id) || 0"
        @select="emit('select', $event)"
      />
    </div>
  </div>
</template>

<style scoped>
.room-list {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 2rem 1rem;
  text-align: center;
}

.empty p {
  font-size: 0.85rem;
  color: var(--text-secondary);
  margin-bottom: 0.25rem;
}

.empty span {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.room-section {
  margin-bottom: 0.75rem;
}

.section-label {
  padding: 0.5rem 0.75rem 0.25rem;
  font-size: 0.65rem;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--text-muted);
  font-weight: 700;
}
</style>
