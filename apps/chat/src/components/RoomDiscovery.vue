<script setup>
import { ROOM_TYPE_INFO } from '../lib/constants.js'
import { formatRelativeTime } from '../lib/utils.js'

const props = defineProps({
  rooms: { type: Array, default: () => [] },
  joinedRoomIds: { type: Set, default: () => new Set() },
})

const emit = defineEmits(['join', 'close'])
</script>

<template>
  <div class="dialog-overlay" @click.self="emit('close')">
    <div class="dialog">
      <div class="dialog-header">
        <h3>Browse Public Channels</h3>
        <button class="close-btn" @click="emit('close')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <div class="dialog-body">
        <div v-if="!rooms.length" class="empty">
          <p>No public channels found</p>
          <span>Create one and make it public!</span>
        </div>

        <div v-else class="room-grid">
          <div v-for="room in rooms" :key="room.id" class="discovery-card">
            <div class="card-top">
              <span class="card-type">{{ (ROOM_TYPE_INFO[room.type] || ROOM_TYPE_INFO.text).label }}</span>
              <span class="card-time">{{ formatRelativeTime(room.createdAt) }}</span>
            </div>
            <h4 class="card-name">{{ room.name }}</h4>
            <p class="card-creator">by {{ room.creatorName || 'Unknown' }}</p>
            <button
              class="join-btn"
              :disabled="joinedRoomIds.has(room.id)"
              @click="emit('join', room.id)"
            >
              {{ joinedRoomIds.has(room.id) ? 'Joined' : 'Join' }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 1rem;
}

.dialog {
  width: 100%;
  max-width: 560px;
  max-height: 80vh;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.dialog-header h3 {
  font-size: 1rem;
  letter-spacing: 0.06em;
}

.close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: none;
  border: none;
  color: var(--text-muted);
  border-radius: 4px;
}

.close-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.close-btn svg {
  width: 16px;
  height: 16px;
}

.dialog-body {
  padding: 1.25rem;
  overflow-y: auto;
}

.empty {
  text-align: center;
  padding: 2rem;
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

.room-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 0.75rem;
}

.discovery-card {
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.card-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-type {
  font-size: 0.65rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--accent2);
}

.card-time {
  font-size: 0.65rem;
  color: var(--text-muted);
}

.card-name {
  font-size: 0.95rem;
  letter-spacing: 0.04em;
}

.card-creator {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.join-btn {
  margin-top: 0.5rem;
  padding: 0.5rem;
  background: var(--accent);
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 0.8rem;
  transition: opacity 0.15s;
}

.join-btn:hover:not(:disabled) {
  opacity: 0.9;
}

.join-btn:disabled {
  background: var(--bg-active);
  color: var(--text-muted);
  cursor: default;
}
</style>
