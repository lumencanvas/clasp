<script setup>
import { ref, computed, onMounted } from 'vue'
import { ROOM_TYPE_INFO } from '../lib/constants.js'
import { useRooms } from '../composables/useRooms.js'

const props = defineProps({
  namespace: { type: String, required: true },
  node: { type: Object, required: true },
  tree: { type: Map, required: true },
  currentRoomId: { type: String, default: null },
  unreadCounts: { type: Map, default: () => new Map() },
  depth: { type: Number, default: 0 },
})

const emit = defineEmits(['select-room', 'join-room'])

const storageKey = `clasp-ns-collapsed:${props.namespace}`
const collapsed = ref(localStorage.getItem(storageKey) === 'true')

function toggleCollapse() {
  collapsed.value = !collapsed.value
  localStorage.setItem(storageKey, collapsed.value)
}

const { joinedRoomIds } = useRooms()

const displayName = computed(() => {
  const parts = props.namespace.split('/')
  return parts[parts.length - 1]
})

const rooms = computed(() => {
  if (!props.node) return []
  return [...props.node.rooms.values()].sort((a, b) => (a.name || '').localeCompare(b.name || ''))
})

const childNamespaces = computed(() => {
  if (!props.node?.children) return []
  return [...props.node.children].sort()
})

const totalRooms = computed(() => {
  let count = props.node?.rooms?.size || 0
  for (const childNs of childNamespaces.value) {
    const childNode = props.tree.get(childNs)
    if (childNode) count += childNode.rooms.size
  }
  return count
})

function getTypeIcon(room) {
  return (ROOM_TYPE_INFO[room.type] || ROOM_TYPE_INFO.text).icon
}

function isJoined(roomId) {
  return joinedRoomIds.value.has(roomId)
}

function handleRoomClick(room) {
  if (isJoined(room.id)) {
    emit('select-room', room.id)
  } else {
    emit('join-room', room.id)
  }
}
</script>

<template>
  <div :class="['ns-group', { collapsed }]" :style="{ '--depth': depth }">
    <button class="ns-header" @click="toggleCollapse">
      <svg
        :class="['chevron', { open: !collapsed }]"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <polyline points="9 18 15 12 9 6"/>
      </svg>
      <span class="ns-name">{{ displayName }}</span>
      <span class="ns-count">{{ totalRooms }}</span>
    </button>

    <div v-if="!collapsed" class="ns-content">
      <!-- Rooms directly in this namespace -->
      <button
        v-for="room in rooms"
        :key="room.id"
        :class="['ns-room', { active: room.id === currentRoomId, dimmed: !isJoined(room.id) }]"
        @click="handleRoomClick(room)"
      >
        <span v-if="getTypeIcon(room) === '#'" class="room-icon">#</span>
        <svg v-else-if="getTypeIcon(room) === 'cam'" class="room-icon-svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polygon points="23 7 16 12 23 17 23 7"/>
          <rect x="1" y="5" width="15" height="14" rx="2" ry="2"/>
        </svg>
        <svg v-else class="room-icon-svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
          <line x1="3" y1="9" x2="21" y2="9"/>
          <line x1="9" y1="21" x2="9" y2="9"/>
        </svg>
        <span class="room-name">{{ room.name }}</span>
        <span v-if="!isJoined(room.id)" class="join-hint">+</span>
        <span
          v-else-if="unreadCounts.get(room.id)"
          class="unread-badge"
        >{{ unreadCounts.get(room.id) > 99 ? '99+' : unreadCounts.get(room.id) }}</span>
      </button>

      <!-- Child namespaces (recursive) -->
      <NamespaceGroup
        v-for="childNs in childNamespaces"
        :key="childNs"
        :namespace="childNs"
        :node="tree.get(childNs)"
        :tree="tree"
        :current-room-id="currentRoomId"
        :unread-counts="unreadCounts"
        :depth="depth + 1"
        @select-room="emit('select-room', $event)"
        @join-room="emit('join-room', $event)"
      />
    </div>
  </div>
</template>

<style scoped>
.ns-group {
  margin-left: calc(var(--depth, 0) * 0.5rem);
}

.ns-header {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  width: 100%;
  padding: 0.35rem 0.5rem;
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 0.7rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  text-align: left;
  border-radius: 4px;
  transition: all 0.1s;
  cursor: pointer;
}

.ns-header:hover {
  color: var(--text-secondary);
  background: var(--bg-tertiary);
}

.chevron {
  width: 12px;
  height: 12px;
  flex-shrink: 0;
  transition: transform 0.15s;
}

.chevron.open {
  transform: rotate(90deg);
}

.ns-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ns-count {
  font-size: 0.6rem;
  color: var(--text-disabled);
  font-weight: 400;
}

.ns-content {
  padding-left: 0.25rem;
}

.ns-room {
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
  min-height: 32px;
}

.ns-room:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.ns-room.active {
  background: var(--bg-active);
  color: var(--text-primary);
}

.ns-room.dimmed {
  opacity: 0.5;
}

.ns-room.dimmed:hover {
  opacity: 0.8;
}

.room-icon {
  font-size: 1rem;
  opacity: 0.5;
  font-weight: 700;
  flex-shrink: 0;
  width: 16px;
  text-align: center;
}

.room-icon-svg {
  width: 14px;
  height: 14px;
  opacity: 0.5;
  flex-shrink: 0;
}

.room-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.join-hint {
  font-size: 0.75rem;
  color: var(--accent3);
  font-weight: 700;
  flex-shrink: 0;
}

.unread-badge {
  background: var(--accent);
  color: white;
  font-size: 0.6rem;
  font-weight: 700;
  padding: 0.05rem 0.35rem;
  border-radius: 10px;
  min-width: 16px;
  text-align: center;
  flex-shrink: 0;
}
</style>
