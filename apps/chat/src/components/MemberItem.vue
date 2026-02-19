<script setup>
import { ref } from 'vue'
import { useClasp } from '../composables/useClasp.js'
import { useIdentity } from '../composables/useIdentity.js'
import { useKeyVerification } from '../composables/useKeyVerification.js'
import UserAvatar from './UserAvatar.vue'

const props = defineProps({
  member: { type: Object, required: true },
  roomId: { type: String, default: null },
  isAdmin: { type: Boolean, default: false },
  memberIsAdmin: { type: Boolean, default: false },
  isCreator: { type: Boolean, default: false },
})

const emit = defineEmits(['view-profile', 'kick', 'ban'])

const { sessionId } = useClasp()
const { userId } = useIdentity()
const { getStoredFingerprint } = useKeyVerification()

const showFingerprint = ref(false)
const fingerprint = ref(null)

async function viewSafetyNumber() {
  if (!props.roomId || !props.member.id) return
  fingerprint.value = await getStoredFingerprint(props.roomId, props.member.id)
  showFingerprint.value = true
}
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
    <span v-if="isCreator" class="role-badge creator">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="10" height="10">
        <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
      </svg>
    </span>
    <span v-else-if="memberIsAdmin" class="role-badge admin">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="10" height="10">
        <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
      </svg>
    </span>
    <span v-if="member.id === sessionId || member.id === userId" class="you-tag">you</span>
    <div class="member-actions" @click.stop>
      <button v-if="roomId && member.id !== userId" class="action-btn" title="View safety number" aria-label="View safety number" @click="viewSafetyNumber">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="12" height="12">
          <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
          <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
        </svg>
      </button>
      <button v-if="isAdmin && member.id !== sessionId && member.id !== userId" class="action-btn action-danger" title="Kick" aria-label="Kick member" @click="emit('kick', member.id)">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="12" height="12">
          <polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/>
        </svg>
      </button>
    </div>
    <div v-if="showFingerprint" class="fingerprint-overlay" @click="showFingerprint = false">
      <div class="fingerprint-card" @click.stop>
        <h4>Safety Number</h4>
        <p class="fingerprint-user">{{ member.name }}</p>
        <code v-if="fingerprint" class="fingerprint-value">{{ fingerprint }}</code>
        <p v-else class="fingerprint-none">No key fingerprint stored yet.</p>
        <button class="fingerprint-close" @click="showFingerprint = false">Close</button>
      </div>
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

.role-badge {
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.role-badge.creator {
  color: var(--accent);
}

.role-badge.admin {
  color: var(--accent3);
}

.you-tag {
  font-size: 0.75rem;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  flex-shrink: 0;
}

.member-actions {
  display: flex;
  gap: 2px;
  opacity: 1;
}

@media (hover: hover) and (pointer: fine) {
  .member-actions {
    opacity: 0;
    transition: opacity 0.1s;
  }

  .member-item:hover .member-actions {
    opacity: 1;
  }
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  background: transparent;
  border: none;
  border-radius: 3px;
  color: var(--text-muted);
  cursor: pointer;
}

@media (hover: hover) and (pointer: fine) {
  .action-btn {
    width: 28px;
    height: 28px;
  }
}

.action-btn:hover {
  background: var(--bg-active);
  color: var(--text-primary);
}

.action-btn:active {
  transform: scale(0.96);
  opacity: 0.8;
}

.action-btn.action-danger:hover {
  color: var(--danger);
}

.fingerprint-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
}

.fingerprint-card {
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 1.25rem;
  width: min(320px, 90vw);
  text-align: center;
}

.fingerprint-card h4 {
  font-family: var(--font-heading);
  font-size: 0.95rem;
  margin: 0 0 0.5rem;
}

.fingerprint-user {
  font-size: 0.8rem;
  color: var(--text-secondary);
  margin: 0 0 0.75rem;
}

.fingerprint-value {
  display: block;
  font-family: var(--font-code);
  font-size: 0.75rem;
  color: var(--accent);
  background: var(--bg-active);
  padding: 0.5rem;
  border-radius: 4px;
  word-break: break-all;
  line-height: 1.6;
  margin-bottom: 0.75rem;
}

.fingerprint-none {
  font-size: 0.8rem;
  color: var(--text-muted);
  margin-bottom: 0.75rem;
}

.fingerprint-close {
  padding: 0.4rem 1rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 4px;
  color: var(--text-secondary);
  font-size: 0.8rem;
  cursor: pointer;
}

.fingerprint-close:hover {
  color: var(--text-primary);
}
</style>
