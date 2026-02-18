<script setup>
import { computed } from 'vue'
import { ROOM_TYPE_INFO } from '../lib/constants.js'

const props = defineProps({
  roomName: { type: String, default: '' },
  roomType: { type: String, default: 'text' },
  onlineCount: { type: Number, default: 0 },
  showMembers: { type: Boolean, default: false },
})

const emit = defineEmits(['toggle-members', 'toggle-sidebar'])

const typeInfo = computed(() => ROOM_TYPE_INFO[props.roomType] || ROOM_TYPE_INFO.text)
</script>

<template>
  <header class="app-header">
    <div class="header-left">
      <button class="hamburger-btn" aria-label="Toggle sidebar" @click="emit('toggle-sidebar')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="3" y1="6" x2="21" y2="6"/>
          <line x1="3" y1="12" x2="21" y2="12"/>
          <line x1="3" y1="18" x2="21" y2="18"/>
        </svg>
      </button>
      <div class="room-info" v-if="roomName">
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
        <h2 class="room-name">{{ roomName }}</h2>
        <span class="online-count">{{ onlineCount }} online</span>
      </div>
    </div>
    <div class="header-right">
      <button
        class="header-btn"
        :class="{ active: showMembers }"
        @click="emit('toggle-members')"
        title="Toggle members"
        aria-label="Toggle members"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
          <circle cx="9" cy="7" r="4"/>
          <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
          <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
        </svg>
      </button>
    </div>
  </header>
</template>

<style scoped>
.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 1rem;
  height: 52px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  min-width: 0;
}

.hamburger-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background: none;
  border: none;
  color: var(--text-secondary);
  border-radius: 4px;
}

.hamburger-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.room-info {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-width: 0;
}

.room-icon {
  font-size: 1.2rem;
  color: var(--text-muted);
  font-weight: 700;
}

.room-icon-svg {
  width: 18px;
  height: 18px;
  color: var(--text-muted);
  flex-shrink: 0;
}

.room-name {
  font-size: 1rem;
  letter-spacing: 0.04em;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.online-count {
  font-size: 0.7rem;
  color: var(--text-muted);
  white-space: nowrap;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.header-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background: none;
  border: none;
  color: var(--text-secondary);
  border-radius: 4px;
  transition: all 0.15s;
}

.header-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.header-btn.active {
  color: var(--accent);
}

.header-btn svg {
  width: 18px;
  height: 18px;
}

@media (min-width: 768px) {
  .hamburger-btn {
    display: none;
  }
}
</style>
