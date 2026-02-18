<script setup>
import { useClasp } from '../composables/useClasp.js'
import { useIdentity } from '../composables/useIdentity.js'
import UserAvatar from './UserAvatar.vue'

const props = defineProps({
  member: { type: Object, required: true },
  isAdmin: { type: Boolean, default: false },
})

const emit = defineEmits(['view-profile', 'kick', 'ban'])

const { sessionId } = useClasp()
const { userId } = useIdentity()
</script>

<template>
  <div
    :class="['member-item', { self: member.id === sessionId || member.id === userId }]"
    @click="emit('view-profile', member)"
  >
    <UserAvatar
      :name="member.name"
      :color="member.avatarColor"
      :size="28"
      :status="member.status"
      :show-status="true"
    />
    <span class="member-name">{{ member.name }}</span>
    <span v-if="member.id === sessionId || member.id === userId" class="you-tag">you</span>
    <div v-if="isAdmin && member.id !== sessionId && member.id !== userId" class="admin-actions" @click.stop>
      <button class="admin-btn" title="Kick" @click="emit('kick', member.id)">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="12" height="12">
          <polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/>
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.member-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.35rem 0.5rem;
  border-radius: 4px;
  transition: background 0.1s;
  cursor: pointer;
}

.member-item:hover {
  background: var(--bg-tertiary);
}

.member-item.self {
  background: rgba(230,57,70,0.06);
}

.member-name {
  flex: 1;
  font-size: 0.8rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-secondary);
}

.you-tag {
  font-size: 0.6rem;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  flex-shrink: 0;
}

.admin-actions {
  display: flex;
  gap: 2px;
  opacity: 0;
  transition: opacity 0.1s;
}

.member-item:hover .admin-actions {
  opacity: 1;
}

.admin-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  background: transparent;
  border: none;
  border-radius: 3px;
  color: var(--text-muted);
  cursor: pointer;
}

.admin-btn:hover {
  background: var(--bg-active);
  color: var(--danger);
}
</style>
