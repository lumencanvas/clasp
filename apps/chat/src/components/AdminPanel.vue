<script setup>
import { ref, computed, toRef, onMounted, watch } from 'vue'
import { useAdmin } from '../composables/useAdmin.js'
import { useIdentity } from '../composables/useIdentity.js'
import { hashPassword, generateSalt } from '../lib/crypto.js'

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
  bannedUsersList,
  roomMeta,
  subscribeAdmins,
  subscribeRoomMeta,
  promoteToAdmin,
  demoteAdmin,
  isUserAdmin,
  kickUser,
  banUser,
  subscribeBans,
  updateRoomName,
  togglePublic,
  updatePassword,
  unbanUser,
} = useAdmin(roomIdRef)

const editName = ref('')
const newPassword = ref('')
const showPasswordInput = ref(false)
const copySuccess = ref(false)

const inviteLink = computed(() => {
  return `${window.location.origin}/chat?join=${props.roomId}`
})

onMounted(() => {
  subscribeAdmins()
  subscribeBans()
  subscribeRoomMeta()
})

// Populate editName when room meta arrives
watch(roomMeta, (meta) => {
  if (meta?.name && !editName.value) {
    editName.value = meta.name
  }
}, { immediate: true })

function handleCopyLink() {
  navigator.clipboard.writeText(inviteLink.value)
  copySuccess.value = true
  setTimeout(() => { copySuccess.value = false }, 2000)
}

function handleSaveName() {
  if (editName.value.trim()) {
    updateRoomName(editName.value.trim())
  }
}

function handleTogglePublic() {
  togglePublic(!roomMeta.value?.isPublic)
}

async function handleSetPassword() {
  if (!newPassword.value) return
  const salt = generateSalt()
  const hash = await hashPassword(newPassword.value, salt)
  updatePassword(hash, salt)
  newPassword.value = ''
  showPasswordInput.value = false
}

function handleRemovePassword() {
  updatePassword(null, null)
}
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

    <!-- Invite Section (all members) -->
    <div class="section">
      <div class="section-label">Invite Link</div>
      <div class="invite-row">
        <input
          class="invite-input"
          :value="inviteLink"
          readonly
          @click="$event.target.select()"
        />
        <button class="copy-btn" @click="handleCopyLink">
          {{ copySuccess ? 'Copied' : 'Copy' }}
        </button>
      </div>
      <div class="room-id-hint">Room ID: {{ roomId }}</div>
    </div>

    <div v-if="!isAdmin" class="no-perms">
      You do not have admin permissions in this room.
    </div>

    <div v-else class="admin-content">
      <!-- Room Settings Section -->
      <div class="section">
        <div class="section-label">Room Settings</div>

        <!-- Room name -->
        <div class="setting-row">
          <label class="setting-label">Room Name</label>
          <div class="setting-input-row">
            <input
              v-model="editName"
              type="text"
              class="setting-input"
              maxlength="32"
              @keydown.enter="handleSaveName"
            />
            <button class="save-btn" @click="handleSaveName">Save</button>
          </div>
        </div>

        <!-- Public/Private toggle (creator only) -->
        <div v-if="isRoomCreator" class="setting-row">
          <label class="setting-label">Visibility</label>
          <div class="toggle-row">
            <button
              :class="['toggle', { active: roomMeta?.isPublic }]"
              @click="handleTogglePublic"
            >
              <span class="toggle-knob"></span>
            </button>
            <span class="toggle-hint">{{ roomMeta?.isPublic ? 'Public (visible in Browse)' : 'Private (invite-only)' }}</span>
          </div>
        </div>

        <!-- Password (creator only) -->
        <div v-if="isRoomCreator" class="setting-row">
          <label class="setting-label">Password</label>
          <div v-if="roomMeta?.passwordHash" class="password-status">
            <span class="status-indicator set">Password set</span>
            <button class="admin-action" @click="showPasswordInput = !showPasswordInput">Change</button>
            <button class="admin-action ban" @click="handleRemovePassword">Remove</button>
          </div>
          <div v-else class="password-status">
            <span class="status-indicator">No password</span>
            <button class="admin-action promote" @click="showPasswordInput = !showPasswordInput">Set Password</button>
          </div>
          <div v-if="showPasswordInput" class="setting-input-row" style="margin-top: 0.4rem">
            <input
              v-model="newPassword"
              type="password"
              class="setting-input"
              placeholder="Enter new password"
              @keydown.enter="handleSetPassword"
            />
            <button class="save-btn" @click="handleSetPassword">Set</button>
          </div>
        </div>
      </div>

      <!-- Member Management -->
      <div class="section">
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
            <button class="admin-action ban" @click="banUser(member.id, member.name)" title="Ban">
              Ban
            </button>
          </div>
        </div>
      </div>

      <!-- Ban List -->
      <div v-if="bannedUsersList.length" class="section">
        <div class="section-label">Banned Users</div>
        <div class="member-actions" v-for="ban in bannedUsersList" :key="ban.id">
          <span class="member-name">
            {{ ban.name || ban.id }}
          </span>
          <div class="action-btns">
            <button class="admin-action promote" @click="unbanUser(ban.id)" title="Unban">
              Unban
            </button>
          </div>
        </div>
      </div>

      <!-- Danger Zone (creator) -->
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
  max-height: 60vh;
  overflow-y: auto;
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

.section {
  margin-bottom: 1rem;
  padding-bottom: 1rem;
  border-bottom: 1px solid var(--border);
}

.section:last-child {
  border-bottom: none;
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

/* Invite */
.invite-row {
  display: flex;
  gap: 0.35rem;
}

.invite-input {
  flex: 1;
  padding: 0.4rem 0.6rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 0.7rem;
  color: var(--text-secondary);
  min-width: 0;
}

.copy-btn {
  padding: 0.4rem 0.75rem;
  font-size: 0.7rem;
  background: var(--accent);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  white-space: nowrap;
  transition: opacity 0.15s;
}

.copy-btn:hover {
  opacity: 0.9;
}

.room-id-hint {
  font-size: 0.6rem;
  color: var(--text-muted);
  margin-top: 0.3rem;
}

/* Settings */
.setting-row {
  margin-bottom: 0.75rem;
}

.setting-label {
  display: block;
  font-size: 0.7rem;
  color: var(--text-secondary);
  margin-bottom: 0.3rem;
}

.setting-input-row {
  display: flex;
  gap: 0.35rem;
}

.setting-input {
  flex: 1;
  padding: 0.4rem 0.6rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 0.8rem;
  color: var(--text-primary);
  min-width: 0;
}

.setting-input:focus {
  outline: none;
  border-color: var(--accent);
}

.save-btn {
  padding: 0.4rem 0.75rem;
  font-size: 0.7rem;
  background: var(--accent3);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  white-space: nowrap;
  transition: opacity 0.15s;
}

.save-btn:hover {
  opacity: 0.9;
}

/* Toggle */
.toggle-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.toggle {
  width: 40px;
  height: 22px;
  background: var(--bg-active);
  border: none;
  border-radius: 11px;
  position: relative;
  transition: background 0.2s;
  flex-shrink: 0;
  cursor: pointer;
}

.toggle.active {
  background: var(--success);
}

.toggle-knob {
  position: absolute;
  top: 3px;
  left: 3px;
  width: 16px;
  height: 16px;
  background: white;
  border-radius: 50%;
  transition: transform 0.2s;
}

.toggle.active .toggle-knob {
  transform: translateX(18px);
}

.toggle-hint {
  font-size: 0.75rem;
  color: var(--text-muted);
}

/* Password */
.password-status {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.status-indicator {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.status-indicator.set {
  color: var(--accent3);
}

/* Members */
.member-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.35rem 0;
  border-bottom: 1px solid var(--border);
}

.member-actions:last-child {
  border-bottom: none;
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
  margin-top: 1rem;
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
