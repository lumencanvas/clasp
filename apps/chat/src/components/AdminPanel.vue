<script setup>
import { toRef } from 'vue'
import { useAdmin } from '../composables/useAdmin.js'

const props = defineProps({
  roomId: { type: String, required: true },
  members: { type: Array, default: () => [] },
})

const emit = defineEmits(['close'])

const roomIdRef = toRef(props, 'roomId')
const { isRoomCreator, kickUser, banUser } = useAdmin(roomIdRef)
</script>

<template>
  <div class="admin-panel">
    <div class="admin-header">
      <h3>Room Settings</h3>
      <button class="close-btn" @click="emit('close')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16">
          <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>

    <div v-if="!isRoomCreator" class="no-perms">
      You are not the room creator. Admin actions are not available.
    </div>

    <div v-else class="admin-content">
      <div class="section-label">Member Management</div>
      <div class="member-actions" v-for="member in members" :key="member.id">
        <span class="member-name">{{ member.name }}</span>
        <div class="action-btns">
          <button class="admin-action kick" @click="kickUser(member.id)" title="Kick">
            Kick
          </button>
          <button class="admin-action ban" @click="banUser(member.id)" title="Ban">
            Ban
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.admin-panel {
  padding: 1rem;
}

.admin-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1rem;
}

.admin-header h3 {
  font-size: 0.85rem;
  letter-spacing: 0.06em;
}

.close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
}

.close-btn:hover {
  background: var(--bg-active);
  color: var(--text-primary);
}

.no-perms {
  font-size: 0.8rem;
  color: var(--text-muted);
  padding: 1rem 0;
}

.section-label {
  font-size: 0.65rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--text-muted);
  font-weight: 700;
  margin-bottom: 0.5rem;
}

.member-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.35rem 0;
  border-bottom: 1px solid var(--border);
}

.member-name {
  font-size: 0.8rem;
  color: var(--text-secondary);
}

.action-btns {
  display: flex;
  gap: 0.25rem;
}

.admin-action {
  padding: 0.2rem 0.5rem;
  font-size: 0.65rem;
  border: 1px solid var(--border);
  border-radius: 3px;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.1s;
}

.admin-action:hover {
  color: var(--text-primary);
}

.admin-action.kick:hover {
  border-color: var(--accent4);
  color: var(--accent4);
}

.admin-action.ban:hover {
  border-color: var(--danger);
  color: var(--danger);
}
</style>
