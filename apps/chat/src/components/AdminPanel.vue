<script setup>
import { toRef, onMounted } from 'vue'
import { useAdmin } from '../composables/useAdmin.js'
import { useIdentity } from '../composables/useIdentity.js'

const props = defineProps({
  roomId: { type: String, required: true },
  members: { type: Array, default: () => [] },
})

const emit = defineEmits(['close', 'delete-room'])

const roomIdRef = toRef(props, 'roomId')
const { userId } = useIdentity()
const {
  isRoomCreator,
  isAdmin,
  adminList,
  subscribeAdmins,
  promoteToAdmin,
  demoteAdmin,
  isUserAdmin,
  kickUser,
  banUser,
} = useAdmin(roomIdRef)

onMounted(() => {
  subscribeAdmins()
})
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

    <div v-if="!isAdmin" class="no-perms">
      You do not have admin permissions in this room.
    </div>

    <div v-else class="admin-content">
      <div class="section-label">Member Management</div>
      <div class="member-actions" v-for="member in members" :key="member.id">
        <span class="member-name">
          {{ member.name }}
          <span v-if="member.id === (isRoomCreator ? userId : '')" class="role-badge creator">creator</span>
          <span v-else-if="isUserAdmin(member.id)" class="role-badge admin">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="10" height="10">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
            </svg>
            admin
          </span>
        </span>
        <div class="action-btns" v-if="member.id !== userId">
          <!-- Creator-only: promote/demote -->
          <button
            v-if="isRoomCreator && !isUserAdmin(member.id)"
            class="admin-action promote"
            @click="promoteToAdmin(member.id)"
            title="Promote to Admin"
          >Promote</button>
          <button
            v-if="isRoomCreator && isUserAdmin(member.id) && member.id !== userId"
            class="admin-action demote"
            @click="demoteAdmin(member.id)"
            title="Demote Admin"
          >Demote</button>
          <!-- Admin actions: kick/ban -->
          <button class="admin-action kick" @click="kickUser(member.id)" title="Kick">
            Kick
          </button>
          <button class="admin-action ban" @click="banUser(member.id)" title="Ban">
            Ban
          </button>
        </div>
      </div>

      <div v-if="isRoomCreator" class="danger-zone">
        <div class="section-label">Danger Zone</div>
        <button class="delete-room-btn" @click="emit('delete-room', roomId)">
          Delete Room
        </button>
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
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.role-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.2rem;
  font-size: 0.6rem;
  padding: 0.1rem 0.35rem;
  border-radius: 3px;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  font-weight: 700;
}

.role-badge.creator {
  background: rgba(230,57,70,0.15);
  color: var(--accent);
}

.role-badge.admin {
  background: rgba(42,157,143,0.15);
  color: var(--accent3);
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

.admin-action.promote:hover {
  border-color: var(--accent3);
  color: var(--accent3);
}

.admin-action.demote:hover {
  border-color: var(--accent4);
  color: var(--accent4);
}

.danger-zone {
  margin-top: 1.5rem;
  padding-top: 1rem;
  border-top: 1px solid var(--border);
}

.delete-room-btn {
  width: 100%;
  padding: 0.5rem 1rem;
  font-size: 0.8rem;
  background: transparent;
  border: 1px solid var(--danger);
  border-radius: 4px;
  color: var(--danger);
  cursor: pointer;
  transition: all 0.15s;
}

.delete-room-btn:hover {
  background: var(--danger);
  color: white;
}
</style>
