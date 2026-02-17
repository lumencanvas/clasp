<script setup>
import { useIdentity } from '../composables/useIdentity.js'
import { useClasp } from '../composables/useClasp.js'
import RoomList from './RoomList.vue'
import UserAvatar from './UserAvatar.vue'

const props = defineProps({
  rooms: { type: Array, default: () => [] },
  currentRoomId: { type: String, default: null },
  unreadCounts: { type: Map, default: () => new Map() },
  connected: { type: Boolean, default: false },
})

const emit = defineEmits(['select-room', 'create-room', 'browse-rooms'])

const { displayName, avatarColor } = useIdentity()
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-top">
      <h2 class="sidebar-title">CLASP Chat</h2>
      <div class="sidebar-actions">
        <button class="action-btn" @click="emit('create-room')" title="Create channel">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19"/>
            <line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
        </button>
        <button class="action-btn" @click="emit('browse-rooms')" title="Browse channels">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8"/>
            <line x1="21" y1="21" x2="16.65" y2="16.65"/>
          </svg>
        </button>
      </div>
    </div>

    <RoomList
      :rooms="rooms"
      :current-room-id="currentRoomId"
      :unread-counts="unreadCounts"
      @select="emit('select-room', $event)"
    />

    <div class="user-bar">
      <UserAvatar :name="displayName" :color="avatarColor" :size="32" />
      <div class="user-info">
        <span class="user-name">{{ displayName }}</span>
        <span class="user-status">
          <span :class="['status-dot', { online: connected }]"></span>
          {{ connected ? 'Connected' : 'Disconnected' }}
        </span>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  display: flex;
  flex-direction: column;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border);
  height: 100%;
}

.sidebar-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.sidebar-title {
  font-size: 0.85rem;
  letter-spacing: 0.12em;
}

.sidebar-actions {
  display: flex;
  gap: 0.25rem;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: none;
  border: none;
  color: var(--text-muted);
  border-radius: 4px;
  transition: all 0.15s;
}

.action-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.action-btn svg {
  width: 16px;
  height: 16px;
}

.user-bar {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.75rem 1rem;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}

.user-info {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.user-name {
  font-size: 0.8rem;
  font-weight: 700;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.user-status {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  font-size: 0.65rem;
  color: var(--text-muted);
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--danger);
  flex-shrink: 0;
}

.status-dot.online {
  background: var(--success);
}
</style>
