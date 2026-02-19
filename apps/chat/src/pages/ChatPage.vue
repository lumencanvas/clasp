<script setup>
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useClasp } from '../composables/useClasp.js'
import { useIdentity } from '../composables/useIdentity.js'
import { useRooms } from '../composables/useRooms.js'
import { useNotifications } from '../composables/useNotifications.js'
import { useFriends } from '../composables/useFriends.js'
import { ROOM_TYPES } from '../lib/constants.js'
import { useStorage } from '../composables/useStorage.js'
import { useCrypto } from '../composables/useCrypto.js'
import { hashPassword, generateSalt } from '../lib/crypto.js'
import AppLayout from '../components/AppLayout.vue'
import AppHeader from '../components/AppHeader.vue'
import AppSidebar from '../components/AppSidebar.vue'
import MemberList from '../components/MemberList.vue'
import FriendList from '../components/FriendList.vue'
import ChatView from '../components/ChatView.vue'
import VideoChannelView from '../components/VideoChannelView.vue'
import ComboChannelView from '../components/ComboChannelView.vue'
import RoomCreateDialog from '../components/RoomCreateDialog.vue'
import RoomDiscovery from '../components/RoomDiscovery.vue'
import UserProfilePopup from '../components/UserProfilePopup.vue'
import StatusPicker from '../components/StatusPicker.vue'

const router = useRouter()
const route = useRoute()
const { connected, disconnect } = useClasp()
const { displayName, status, setStatus, announceProfile } = useIdentity()
const {
  joinedRooms,
  joinedRoomIds,
  dmRooms,
  currentRoomId,
  currentRoom,
  discoveredRooms,
  createRoom,
  createDM,
  joinRoom,
  leaveRoom,
  deleteRoom,
  fetchRoomMeta,
  updateRoomData,
  switchRoom,
  discoverPublicRooms,
  stopDiscovery,
} = useRooms()
const { unreadCounts, markRead, requestPermission } = useNotifications()
const {
  friendList,
  pendingRequests,
  requestCount,
  init: initFriends,
  cleanup: cleanupFriends,
  acceptRequest,
  declineRequest,
  removeFriend,
} = useFriends()

const { exportData, importData } = useStorage()
const { enableEncryption } = useCrypto()
const importFileRef = ref(null)

const showMembers = ref(true)
const showCreateDialog = ref(false)
const showBrowseDialog = ref(false)
const showFriends = ref(false)
const showStatusPicker = ref(false)
const profileTarget = ref(null) // { userId, name, avatarColor, status }
const layoutRef = ref(null)
const chatViewRef = ref(null)
const videoViewRef = ref(null)
const comboViewRef = ref(null)

// Invite join state
const inviteJoinRoom = ref(null) // room meta when password is needed
const invitePasswordInput = ref('')
const inviteError = ref(null)

async function handleInviteJoin(roomId) {
  // Already joined — just switch
  if (joinedRoomIds.value.has(roomId)) {
    switchRoom(roomId)
    router.replace({ path: '/chat' })
    return
  }

  const meta = await fetchRoomMeta(roomId)
  if (!meta) {
    // Room doesn't exist
    router.replace({ path: '/chat' })
    return
  }

  if (meta.passwordHash) {
    // Password required — show prompt
    inviteJoinRoom.value = { id: roomId, ...meta }
    return
  }

  // No password — join directly
  updateRoomData(roomId, { name: meta.name, type: meta.type, isPublic: meta.isPublic, creatorId: meta.creatorId })
  joinRoom(roomId)
  switchRoom(roomId)
  router.replace({ path: '/chat' })
}

async function handleInvitePassword() {
  if (!inviteJoinRoom.value || !invitePasswordInput.value) return
  inviteError.value = null
  const room = inviteJoinRoom.value

  const hash = await hashPassword(invitePasswordInput.value, room.passwordSalt)
  if (hash !== room.passwordHash) {
    inviteError.value = 'Incorrect password'
    return
  }

  // Publish password proof
  const { userId: uid } = useIdentity()
  const { set } = useClasp()
  set(`/chat/room/${room.id}/crypto/proof/${uid.value}`, {
    hash,
    timestamp: Date.now(),
  })

  updateRoomData(room.id, { name: room.name, type: room.type, isPublic: room.isPublic })
  joinRoom(room.id)
  switchRoom(room.id)
  inviteJoinRoom.value = null
  invitePasswordInput.value = ''
  router.replace({ path: '/chat' })
}

function cancelInviteJoin() {
  inviteJoinRoom.value = null
  invitePasswordInput.value = ''
  inviteError.value = null
  router.replace({ path: '/chat' })
}

const activeViewRef = computed(() => {
  if (!currentRoom.value) return null
  const type = currentRoom.value.type
  if (type === ROOM_TYPES.TEXT || type === ROOM_TYPES.DM) return chatViewRef.value
  if (type === ROOM_TYPES.VIDEO) return videoViewRef.value
  if (type === ROOM_TYPES.COMBO) return comboViewRef.value
  return null
})

const activeMemberList = computed(() => activeViewRef.value?.sortedParticipants ?? [])
const activeOnlineCount = computed(() => activeViewRef.value?.onlineCount ?? 0)

// Redirect if not connected (preserve ?join param across the redirect)
watch(connected, (val) => {
  if (!val) {
    const joinParam = route.query.join
    router.push(joinParam ? { path: '/', query: { join: joinParam } } : '/')
  }
}, { immediate: true })

// Mark room as read when switching
watch(currentRoomId, (roomId) => {
  if (roomId) markRead(roomId)
})

function handleSelectRoom(roomId) {
  switchRoom(roomId)
  layoutRef.value?.closeSidebar()
}

async function handleCreateRoom(opts) {
  // Hash password if provided
  let passwordHash = null
  let passwordSalt = null
  if (opts.password) {
    passwordSalt = generateSalt()
    passwordHash = await hashPassword(opts.password, passwordSalt)
  }

  const roomId = createRoom({
    name: opts.name,
    type: opts.type,
    isPublic: opts.isPublic,
    encrypted: opts.encrypted,
    passwordHash,
    passwordSalt,
  })
  if (roomId) {
    // Enable E2E encryption if requested
    if (opts.encrypted) {
      await enableEncryption(roomId)
    }
    switchRoom(roomId)
    showCreateDialog.value = false
  }
}

async function handleBrowseJoin(roomId, password) {
  // If room has a password, verify it before joining
  if (password) {
    const { subscribe } = useClasp()
    // Fetch room meta to get password hash/salt
    const meta = await new Promise((resolve) => {
      const unsub = subscribe(`/chat/room/${roomId}/meta`, (data) => {
        resolve(data)
        unsub()
      })
      // Timeout after 3s
      setTimeout(() => resolve(null), 3000)
    })

    if (meta && meta.passwordHash && meta.passwordSalt) {
      const hash = await hashPassword(password, meta.passwordSalt)
      if (hash !== meta.passwordHash) {
        // Wrong password — don't join
        return
      }
      // Correct password — publish password proof for crypto gating
      const { userId: uid } = useIdentity()
      const { set } = useClasp()
      set(`/chat/room/${roomId}/crypto/proof/${uid.value}`, {
        hash,
        timestamp: Date.now(),
      })
    }
  }

  joinRoom(roomId)
  switchRoom(roomId)
  showBrowseDialog.value = false
}

function handleBrowse() {
  discoverPublicRooms()
  showBrowseDialog.value = true
}

function handleOpenDM(targetUserId, targetName) {
  const roomId = createDM(targetUserId, targetName)
  if (roomId) {
    switchRoom(roomId)
    profileTarget.value = null
    showFriends.value = false
  }
}

function handleFriendMessage(friend) {
  handleOpenDM(friend.id, friend.name)
}

function handleViewProfile(member) {
  profileTarget.value = {
    userId: member.id,
    name: member.name,
    avatarColor: member.avatarColor,
    status: member.status,
  }
}

function handleStatusChange(newStatus) {
  setStatus(newStatus)
  showStatusPicker.value = false
}

function handleDeleteRoom(roomId) {
  if (confirm('Delete this room? This cannot be undone.')) {
    deleteRoom(roomId)
  }
}

function handleLogout() {
  cleanupFriends()
  disconnect()
  localStorage.removeItem('clasp-chat-token')
  localStorage.removeItem('clasp-chat-auth-userId')
  localStorage.removeItem('clasp-chat-auth-username')
  router.push('/')
}

function handleExport() { exportData() }
function handleImport() { importFileRef.value?.click() }
async function handleImportFile(e) {
  const file = e.target.files?.[0]
  if (!file) return
  try {
    await importData(file)
  } catch { /* ignore */ }
  e.target.value = ''
}

onMounted(() => {
  announceProfile()
  requestPermission()
  initFriends()

  discoverPublicRooms()

  // Handle invite join from URL
  const joinParam = route.query.join
  if (joinParam) {
    handleInviteJoin(joinParam)
  } else if (!currentRoomId.value && joinedRooms.value.length > 0) {
    switchRoom(joinedRooms.value[0].id)
  }
})

onUnmounted(() => {
  stopDiscovery()
  cleanupFriends()
})
</script>

<template>
  <AppLayout ref="layoutRef" :show-members="showMembers && !!currentRoom && !showFriends">
    <template #sidebar="{ closeSidebar }">
      <AppSidebar
        :rooms="joinedRooms"
        :dm-rooms="dmRooms"
        :current-room-id="currentRoomId"
        :unread-counts="unreadCounts"
        :connected="connected"
        :request-count="requestCount"
        @select-room="handleSelectRoom"
        @select-dm="handleSelectRoom"
        @create-room="showCreateDialog = true"
        @browse-rooms="handleBrowse"
        @toggle-friends="showFriends = !showFriends"
        @status-change="showStatusPicker = true"
        @logout="handleLogout"
      />
    </template>

    <template #header="{ toggleSidebar }">
      <AppHeader
        :room-name="currentRoom?.name"
        :room-type="currentRoom?.type"
        :online-count="activeOnlineCount"
        :show-members="showMembers"
        @toggle-sidebar="toggleSidebar"
        @toggle-members="showMembers = !showMembers"
      />
    </template>

    <!-- Main content: channel view based on room type -->
    <template v-if="currentRoom">
      <ChatView
        v-if="currentRoom.type === ROOM_TYPES.TEXT || currentRoom.type === ROOM_TYPES.DM"
        ref="chatViewRef"
        :room-id="currentRoomId"
        :is-active="true"
        @delete-room="handleDeleteRoom"
      />
      <VideoChannelView
        v-else-if="currentRoom.type === ROOM_TYPES.VIDEO"
        ref="videoViewRef"
        :room-id="currentRoomId"
      />
      <ComboChannelView
        v-else-if="currentRoom.type === ROOM_TYPES.COMBO"
        ref="comboViewRef"
        :room-id="currentRoomId"
      />
    </template>

    <!-- No room selected -->
    <div v-else class="no-room">
      <div class="no-room-content">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
          <path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"/>
        </svg>
        <h2>Welcome, {{ displayName }}</h2>
        <p>Create or join a channel to start chatting</p>
        <div class="no-room-actions">
          <button class="action-btn primary" @click="showCreateDialog = true">
            Create Channel
          </button>
          <button class="action-btn secondary" @click="handleBrowse">
            Browse Public
          </button>
        </div>
        <div class="data-actions">
          <button class="data-btn" @click="handleExport">Export Data</button>
          <button class="data-btn" @click="handleImport">Import Data</button>
          <input type="file" ref="importFileRef" accept=".json" style="display:none" @change="handleImportFile" />
        </div>
      </div>
    </div>

    <template #members>
      <!-- Show friends panel or member list -->
      <FriendList
        v-if="showFriends"
        :friends="friendList"
        :pending-requests="pendingRequests"
        @message="handleFriendMessage"
        @accept="acceptRequest"
        @decline="declineRequest"
        @remove="removeFriend"
        @view-profile="handleViewProfile"
        @close="showFriends = false"
      />
      <MemberList
        v-else
        :members="activeMemberList"
        @view-profile="handleViewProfile"
      />
    </template>
  </AppLayout>

  <!-- Dialogs -->
  <RoomCreateDialog
    v-if="showCreateDialog"
    @create="handleCreateRoom"
    @close="showCreateDialog = false"
  />
  <RoomDiscovery
    v-if="showBrowseDialog"
    :rooms="discoveredRooms"
    :joined-room-ids="joinedRoomIds"
    @join="handleBrowseJoin"
    @close="showBrowseDialog = false"
  />
  <UserProfilePopup
    v-if="profileTarget"
    :user-id="profileTarget.userId"
    :name="profileTarget.name"
    :avatar-color="profileTarget.avatarColor"
    :status="profileTarget.status"
    @close="profileTarget = null"
    @send-dm="handleOpenDM"
  />
  <StatusPicker
    v-if="showStatusPicker"
    :current-status="status"
    @select="handleStatusChange"
    @close="showStatusPicker = false"
  />

  <!-- Invite join password dialog -->
  <div v-if="inviteJoinRoom" class="dialog-overlay" @click.self="cancelInviteJoin">
    <div class="invite-dialog">
      <h3>Join "{{ inviteJoinRoom.name }}"</h3>
      <p class="invite-desc">This room requires a password to join.</p>
      <form @submit.prevent="handleInvitePassword">
        <input
          v-model="invitePasswordInput"
          type="password"
          placeholder="Enter room password"
          class="invite-password-input"
          autofocus
        />
        <p v-if="inviteError" class="invite-error">{{ inviteError }}</p>
        <div class="invite-actions">
          <button type="button" class="action-btn secondary" @click="cancelInviteJoin">Cancel</button>
          <button type="submit" class="action-btn primary" :disabled="!invitePasswordInput">Join</button>
        </div>
      </form>
    </div>
  </div>
</template>

<style scoped>
.no-room {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
}

.no-room-content {
  text-align: center;
  max-width: 360px;
}

.no-room-content svg {
  width: 64px;
  height: 64px;
  color: var(--text-muted);
  margin-bottom: 1.5rem;
}

.no-room-content h2 {
  font-size: 1.2rem;
  letter-spacing: 0.06em;
  margin-bottom: 0.5rem;
}

.no-room-content p {
  font-size: 0.85rem;
  color: var(--text-secondary);
  margin-bottom: 1.5rem;
}

.no-room-actions {
  display: flex;
  gap: 0.75rem;
  justify-content: center;
  flex-wrap: wrap;
}

.action-btn {
  min-height: 44px;
  padding: 0.75rem 1.25rem;
  border: none;
  border-radius: 4px;
  font-size: 0.85rem;
  letter-spacing: 0.04em;
  transition: opacity 0.15s;
}

.action-btn:hover {
  opacity: 0.9;
}

.action-btn.primary {
  background: var(--accent);
  color: white;
}

.action-btn.secondary {
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  color: var(--text-primary);
}

.data-actions {
  display: flex;
  gap: 0.5rem;
  justify-content: center;
  margin-top: 1rem;
}

.data-btn {
  padding: 0.4rem 0.8rem;
  background: transparent;
  border: 1px solid var(--border);
  border-radius: 4px;
  color: var(--text-muted);
  font-size: 0.7rem;
  letter-spacing: 0.04em;
  cursor: pointer;
  transition: all 0.15s;
}

.data-btn:hover {
  border-color: var(--text-muted);
  color: var(--text-secondary);
}

/* Invite join dialog */
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
  padding: 1rem;
}

.invite-dialog {
  width: 100%;
  max-width: 380px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 1.5rem;
}

.invite-dialog h3 {
  font-size: 1rem;
  letter-spacing: 0.06em;
  margin-bottom: 0.5rem;
}

.invite-desc {
  font-size: 0.8rem;
  color: var(--text-secondary);
  margin-bottom: 1rem;
}

.invite-password-input {
  width: 100%;
  padding: 0.75rem 1rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 0.9rem;
  margin-bottom: 0.75rem;
  box-sizing: border-box;
}

.invite-password-input:focus {
  outline: none;
  border-color: var(--accent);
}

.invite-error {
  color: var(--danger);
  font-size: 0.8rem;
  margin-bottom: 0.5rem;
}

.invite-actions {
  display: flex;
  gap: 0.5rem;
  justify-content: flex-end;
}
</style>
