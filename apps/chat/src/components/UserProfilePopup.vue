<script setup>
import { ref, onMounted } from 'vue'
import { useClasp } from '../composables/useClasp.js'
import { useIdentity } from '../composables/useIdentity.js'
import { useFriends } from '../composables/useFriends.js'
import { ADDR } from '../lib/constants.js'
import UserAvatar from './UserAvatar.vue'

const props = defineProps({
  userId: { type: String, required: true },
  // Pre-populated from member data (so we don't need to wait for fetch)
  name: { type: String, default: '' },
  avatarColor: { type: String, default: null },
  status: { type: String, default: null },
})

const emit = defineEmits(['close', 'send-dm'])

const { get } = useClasp()
const { userId: myUserId } = useIdentity()
const { isFriend, sendRequest, removeFriend } = useFriends()

const profile = ref({
  name: props.name,
  avatarColor: props.avatarColor,
  status: props.status,
})
const loading = ref(false)
const isMe = ref(props.userId === myUserId.value)
const friendStatus = ref(isFriend(props.userId) ? 'friend' : 'none')
const requestSent = ref(false)

onMounted(async () => {
  if (isMe.value) return
  loading.value = true
  try {
    const data = await get(`${ADDR.USER_PROFILE}/${props.userId}/profile`)
    if (data) {
      profile.value = { ...profile.value, ...data }
    }
  } catch (e) {
    // Use pre-populated data
  } finally {
    loading.value = false
  }
})

function handleAddFriend() {
  sendRequest(props.userId, profile.value.name)
  requestSent.value = true
}

function handleRemoveFriend() {
  removeFriend(props.userId)
  friendStatus.value = 'none'
}

function handleSendDM() {
  emit('send-dm', props.userId, profile.value.name)
}

const statusLabel = {
  online: 'Online',
  away: 'Away',
  dnd: 'Do Not Disturb',
  invisible: 'Offline',
}
</script>

<template>
  <div class="popup-overlay" @click.self="emit('close')">
    <div class="profile-popup">
      <button class="close-btn" @click="emit('close')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>

      <div class="profile-header">
        <UserAvatar
          :name="profile.name"
          :color="profile.avatarColor"
          :size="64"
          :status="profile.status"
          :show-status="true"
        />
        <h3 class="profile-name">{{ profile.name }}</h3>
        <span v-if="profile.status" class="profile-status">
          {{ statusLabel[profile.status] || profile.status }}
        </span>
        <span v-if="isMe" class="you-tag">This is you</span>
      </div>

      <div v-if="!isMe" class="profile-actions">
        <button class="profile-btn primary" @click="handleSendDM">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"/>
          </svg>
          Message
        </button>

        <button
          v-if="friendStatus === 'friend'"
          class="profile-btn danger"
          @click="handleRemoveFriend"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
            <circle cx="9" cy="7" r="4"/>
            <line x1="18" y1="11" x2="23" y2="11"/>
          </svg>
          Unfriend
        </button>
        <button
          v-else
          class="profile-btn secondary"
          :disabled="requestSent"
          @click="handleAddFriend"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
            <circle cx="9" cy="7" r="4"/>
            <line x1="20" y1="8" x2="20" y2="14"/>
            <line x1="17" y1="11" x2="23" y2="11"/>
          </svg>
          {{ requestSent ? 'Request Sent' : 'Add Friend' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.popup-overlay {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0,0,0,0.5);
  z-index: 200;
}

.profile-popup {
  position: relative;
  width: 280px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 1.5rem;
  box-shadow: 0 8px 32px rgba(0,0,0,0.3);
}

.close-btn {
  position: absolute;
  top: 0.75rem;
  right: 0.75rem;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: transparent;
  border: none;
  color: var(--text-muted);
  border-radius: 4px;
  cursor: pointer;
}

.close-btn svg {
  width: 16px;
  height: 16px;
}

.close-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.profile-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 1.25rem;
}

.profile-name {
  font-size: 1rem;
  font-weight: 700;
  letter-spacing: 0.04em;
}

.profile-status {
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.you-tag {
  font-size: 0.7rem;
  color: var(--text-muted);
  font-style: italic;
}

.profile-actions {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.profile-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.6rem 1rem;
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 0.8rem;
  cursor: pointer;
  transition: all 0.15s;
}

.profile-btn svg {
  width: 16px;
  height: 16px;
}

.profile-btn.primary {
  background: var(--accent);
  border-color: var(--accent);
  color: white;
}

.profile-btn.primary:hover { opacity: 0.9; }

.profile-btn.secondary {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.profile-btn.secondary:hover { background: var(--bg-active); }

.profile-btn.danger {
  background: transparent;
  color: var(--danger);
  border-color: var(--danger);
}

.profile-btn.danger:hover {
  background: rgba(230,57,70,0.1);
}

.profile-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
