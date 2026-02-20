<script setup>
import { computed, onMounted } from 'vue'
import { useIdentity } from '../composables/useIdentity.js'
import { useNamespaces } from '../composables/useNamespaces.js'
import RoomList from './RoomList.vue'
import NamespaceGroup from './NamespaceGroup.vue'
import UserAvatar from './UserAvatar.vue'

const props = defineProps({
  rooms: { type: Array, default: () => [] },
  dmRooms: { type: Array, default: () => [] },
  currentRoomId: { type: String, default: null },
  unreadCounts: { type: Map, default: () => new Map() },
  connected: { type: Boolean, default: false },
  requestCount: { type: Number, default: 0 },
})

const emit = defineEmits([
  'select-room',
  'create-room',
  'browse-rooms',
  'toggle-friends',
  'select-dm',
  'status-change',
  'logout',
])

const { displayName, avatarColor, status } = useIdentity()
const {
  namespaceTree,
  pinnedNamespaceList,
  namespacedRoomIds,
  initPinnedNamespaces,
} = useNamespaces()

// Filter out rooms that are in a pinned namespace
const ungroupedRooms = computed(() => {
  return props.rooms.filter(r => !namespacedRoomIds.value.has(r.id))
})

onMounted(() => {
  if (props.connected) {
    initPinnedNamespaces()
  }
})
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-top">
      <h2 class="sidebar-title">CLASP Chat</h2>
      <div class="sidebar-actions">
        <button
          class="action-btn"
          @click="emit('toggle-friends')"
          title="Friends"
          aria-label="Friends"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
            <circle cx="9" cy="7" r="4"/>
            <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
            <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
          </svg>
          <span v-if="requestCount > 0" class="request-badge">{{ requestCount > 9 ? '9+' : requestCount }}</span>
        </button>
        <button class="action-btn" @click="emit('create-room')" title="Create channel" aria-label="Create channel">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19"/>
            <line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
        </button>
        <button class="action-btn" @click="emit('browse-rooms')" title="Browse channels" aria-label="Browse channels">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8"/>
            <line x1="21" y1="21" x2="16.65" y2="16.65"/>
          </svg>
        </button>
      </div>
    </div>

    <div class="sidebar-scroll">
      <RoomList
        :rooms="ungroupedRooms"
        :current-room-id="currentRoomId"
        :unread-counts="unreadCounts"
        @select="emit('select-room', $event)"
      />

      <!-- Pinned Namespaces -->
      <div v-if="pinnedNamespaceList.length" class="ns-section">
        <div class="section-label">Namespaces</div>
        <NamespaceGroup
          v-for="ns in pinnedNamespaceList"
          :key="ns"
          :namespace="ns"
          :node="namespaceTree.get(ns)"
          :tree="namespaceTree"
          :current-room-id="currentRoomId"
          :unread-counts="unreadCounts"
          @select-room="emit('select-room', $event)"
          @join-room="emit('select-room', $event)"
        />
      </div>

      <!-- DM section -->
      <div v-if="dmRooms.length" class="dm-section">
        <div class="section-label">Direct Messages</div>
        <button
          v-for="dm in dmRooms"
          :key="dm.id"
          :class="['dm-item', { active: dm.id === currentRoomId }]"
          @click="emit('select-dm', dm.id)"
        >
          <UserAvatar :name="dm.name" :size="22" />
          <span class="dm-name">{{ dm.name }}</span>
          <span
            v-if="unreadCounts.get(dm.id)"
            class="unread-badge"
          >{{ unreadCounts.get(dm.id) > 99 ? '99+' : unreadCounts.get(dm.id) }}</span>
        </button>
      </div>
    </div>

    <div class="user-bar">
      <div class="user-bar-info" @click="emit('status-change')">
        <UserAvatar
          :name="displayName"
          :color="avatarColor"
          :size="32"
          :status="status"
          :show-status="true"
        />
        <div class="user-info">
          <span class="user-name">{{ displayName }}</span>
          <span class="user-status">
            <span :class="['status-dot', status]"></span>
            {{ status === 'dnd' ? 'Do Not Disturb' : status.charAt(0).toUpperCase() + status.slice(1) }}
          </span>
        </div>
      </div>
      <button class="action-btn logout-btn" title="Log out" aria-label="Log out" @click="emit('logout')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/>
          <polyline points="16 17 21 12 16 7"/>
          <line x1="21" y1="12" x2="9" y2="12"/>
        </svg>
      </button>
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
  position: relative;
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

.request-badge {
  position: absolute;
  top: 0;
  right: -2px;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  background: var(--danger);
  color: white;
  font-size: 0.6rem;
  font-weight: 700;
  line-height: 16px;
  text-align: center;
  border-radius: 8px;
  border: 1.5px solid var(--bg-secondary);
}

.sidebar-scroll {
  flex: 1;
  overflow-y: auto;
}

.ns-section {
  padding: 0 0.5rem 0.5rem;
}

.dm-section {
  padding: 0 0.5rem 0.5rem;
}

.section-label {
  padding: 0.75rem 0.75rem 0.25rem;
  font-size: 0.65rem;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--text-muted);
  font-weight: 700;
}

.dm-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.4rem 0.75rem;
  background: none;
  border: none;
  color: var(--text-secondary);
  font-size: 0.8rem;
  text-align: left;
  border-radius: 4px;
  transition: all 0.1s;
  min-height: 34px;
}

.dm-item:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.dm-item.active {
  background: var(--bg-active);
  color: var(--text-primary);
}

.dm-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.unread-badge {
  background: var(--accent);
  color: white;
  font-size: 0.65rem;
  font-weight: 700;
  padding: 0.1rem 0.4rem;
  border-radius: 10px;
  min-width: 18px;
  text-align: center;
  flex-shrink: 0;
}

.user-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}

.user-bar-info {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  min-width: 0;
  flex: 1;
  cursor: pointer;
  border-radius: 4px;
  padding: 0.15rem;
  transition: background 0.1s;
}

.user-bar-info:hover {
  background: var(--bg-tertiary);
}

.logout-btn:hover {
  color: var(--danger) !important;
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
  flex-shrink: 0;
}

.status-dot.online { background: var(--success); }
.status-dot.away { background: var(--accent4); }
.status-dot.dnd { background: var(--danger); }
.status-dot.invisible { background: var(--text-disabled); }
</style>
