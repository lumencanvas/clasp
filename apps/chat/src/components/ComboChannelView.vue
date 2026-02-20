<script setup>
import { ref, computed, toRef, onUnmounted } from 'vue'
import { useVideoRoom } from '../composables/useVideoRoom.js'
import { useVideoLayout } from '../composables/useVideoLayout.js'
import { useChat } from '../composables/useChat.js'
import { useReactions } from '../composables/useReactions.js'
import { useIdentity } from '../composables/useIdentity.js'
import { useAdmin } from '../composables/useAdmin.js'
import LocalPreview from './LocalPreview.vue'
import VideoGrid from './VideoGrid.vue'
import VideoControls from './VideoControls.vue'
import MessageList from './MessageList.vue'
import MessageComposer from './MessageComposer.vue'
import TypingIndicator from './TypingIndicator.vue'
import AdminPanel from './AdminPanel.vue'

const props = defineProps({
  roomId: { type: String, required: true },
})

const emit = defineEmits(['delete-room'])

const roomIdRef = toRef(props, 'roomId')
const { displayName, avatarColor } = useIdentity()
const { isAdmin, subscribeBans, subscribeAdmins } = useAdmin(roomIdRef)

const showAdmin = ref(false)

const unsubBans = subscribeBans()
const unsubAdmins = subscribeAdmins()

onUnmounted(() => {
  if (unsubBans) unsubBans()
  if (unsubAdmins) unsubAdmins()
})

// Chat composable
const {
  messages,
  sortedParticipants: chatParticipants,
  typingList,
  replyTo,
  editingMessage,
  sendMessage,
  editMessage,
  deleteMessage,
  setReplyTo,
  startEditing,
  cancelEditing,
  handleTyping,
} = useChat(roomIdRef)

// Reactions composable
const { toggleReaction, getMessageReactions } = useReactions(roomIdRef)

// Video composable
const {
  localStream,
  inVideo,
  audioEnabled,
  videoEnabled,
  isScreenSharing,
  error: videoError,
  peerList,
  participantList,
  speakingPeerIds,
  getUserMedia,
  getUserMediaSelective,
  enableAudio,
  enableVideo,
  joinVideo,
  leaveVideo,
  toggleAudio,
  toggleVideo,
  shareScreen,
  stopUserMedia,
} = useVideoRoom(roomIdRef)

const { layout, pinnedPeerId, spotlightPeer, setLayout, pinPeer, unpinPeer } = useVideoLayout(isScreenSharing, speakingPeerIds)

const mediaLoading = ref(false)
const videoCollapsed = ref(false)

// Resizable split
const splitRatio = ref(typeof window !== 'undefined' && window.innerWidth < 768 ? 40 : 60)
const isResizing = ref(false)
const comboRef = ref(null)

function startResize(e) {
  e.preventDefault()
  isResizing.value = true

  function onMove(ev) {
    const containerEl = comboRef.value
    if (!containerEl) return
    const rect = containerEl.getBoundingClientRect()

    // Detect if horizontal (desktop) or vertical (mobile/tablet)
    if (window.innerWidth >= 1024) {
      const clientX = ev.clientX || (ev.touches && ev.touches[0].clientX)
      const ratio = ((clientX - rect.left) / rect.width) * 100
      splitRatio.value = Math.min(80, Math.max(20, ratio))
    } else {
      const clientY = ev.clientY || (ev.touches && ev.touches[0].clientY)
      const ratio = ((clientY - rect.top) / rect.height) * 100
      splitRatio.value = Math.min(80, Math.max(20, ratio))
    }
  }

  function onUp() {
    isResizing.value = false
    document.removeEventListener('pointermove', onMove)
    document.removeEventListener('pointerup', onUp)
  }

  document.addEventListener('pointermove', onMove)
  document.addEventListener('pointerup', onUp)
}

// Merge chat + video participants for member list
const sortedParticipants = computed(() => {
  const merged = new Map()
  for (const p of chatParticipants.value) {
    merged.set(p.id, p)
  }
  for (const p of participantList.value) {
    if (!merged.has(p.id)) {
      merged.set(p.id, { id: p.id, name: p.name, avatarColor: p.avatarColor })
    }
  }
  return Array.from(merged.values()).sort((a, b) => a.name.localeCompare(b.name))
})

const onlineCount = computed(() => {
  const ids = new Set()
  for (const p of chatParticipants.value) ids.add(p.id)
  for (const p of participantList.value) ids.add(p.id)
  return ids.size + 1
})

defineExpose({ sortedParticipants, onlineCount })

function requestCameraPreview() {
  if (!localStream.value) getUserMedia().catch(() => {})
}

function stopCameraPreview() {
  stopUserMedia()
}

async function handleJoin({ audio, video }) {
  mediaLoading.value = true
  try {
    stopUserMedia()
    await getUserMediaSelective({ audio, video })
    await joinVideo()
  } finally {
    mediaLoading.value = false
  }
}

function handleLeaveVideo() {
  leaveVideo()
  stopUserMedia()
}

async function handleToggleAudio() {
  if (!localStream.value || !localStream.value.getAudioTracks().length) {
    await enableAudio()
  } else {
    toggleAudio()
  }
}

async function handleToggleVideo() {
  if (!localStream.value || !localStream.value.getVideoTracks().length) {
    await enableVideo()
  } else {
    toggleVideo()
  }
}

function handleSend(text) {
  sendMessage(text)
}

function handleSendImage(dataUrl) {
  sendMessage('', { image: dataUrl })
}

function handlePin(id) {
  pinPeer(id)
  if (layout.value === 'grid') {
    setLayout('spotlight')
  }
}

function handleTogglePin() {
  if (pinnedPeerId.value) {
    unpinPeer()
  }
}
</script>

<template>
  <div ref="comboRef" :class="['combo-channel', { resizing: isResizing }]">
    <!-- Admin overlay (positioned absolutely so it doesn't break row layout) -->
    <div v-if="isAdmin" class="admin-overlay-wrap">
      <button
        class="admin-toggle"
        aria-label="Room settings"
        title="Room settings"
        @click="showAdmin = !showAdmin"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="18" height="18">
          <circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
        </svg>
      </button>
      <div v-if="showAdmin" class="admin-panel-dropdown">
        <AdminPanel
          :room-id="roomId"
          :members="sortedParticipants"
          @close="showAdmin = false"
          @delete-room="emit('delete-room', $event)"
        />
      </div>
    </div>

    <!-- Video section -->
    <div
      :class="['combo-video', { collapsed: videoCollapsed }]"
      :style="!videoCollapsed ? { flexBasis: splitRatio + '%' } : undefined"
    >
      <button class="collapse-toggle" @click="videoCollapsed = !videoCollapsed">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline v-if="videoCollapsed" points="6 9 12 15 18 9"/>
          <polyline v-else points="18 15 12 9 6 15"/>
        </svg>
        {{ videoCollapsed ? 'Show Video' : 'Hide Video' }}
      </button>

      <div v-if="!videoCollapsed" class="video-area">
        <LocalPreview
          v-if="!inVideo"
          :stream="localStream"
          :loading="mediaLoading"
          :error="videoError"
          @join="handleJoin"
          @request-camera="requestCameraPreview"
          @stop-camera="stopCameraPreview"
        />
        <template v-else>
          <VideoGrid
            :local-stream="localStream"
            :local-name="displayName"
            :audio-enabled="audioEnabled"
            :video-enabled="videoEnabled"
            :is-screen-share="isScreenSharing"
            :avatar-color="avatarColor"
            :peers="peerList"
            :layout="layout"
            :spotlight-peer="spotlightPeer"
            :pinned-peer-id="pinnedPeerId"
            :speaking-peer-ids="speakingPeerIds"
            @pin="handlePin"
          />
          <VideoControls
            :audio-enabled="audioEnabled"
            :video-enabled="videoEnabled"
            :is-screen-sharing="isScreenSharing"
            :layout="layout"
            :has-pinned-peer="!!pinnedPeerId"
            @toggle-audio="handleToggleAudio"
            @toggle-video="handleToggleVideo"
            @share-screen="shareScreen"
            @set-layout="setLayout"
            @toggle-pin="handleTogglePin"
            @leave="handleLeaveVideo"
          />
        </template>
      </div>
    </div>

    <!-- Resize handle -->
    <div
      v-if="!videoCollapsed"
      class="resize-handle"
      @pointerdown="startResize"
    ></div>

    <!-- Chat section -->
    <div class="combo-chat">
      <MessageList
        :messages="messages"
        :get-reactions="getMessageReactions"
        @reply="setReplyTo"
        @edit="startEditing"
        @delete="deleteMessage"
        @react="toggleReaction"
      />
      <TypingIndicator :users="typingList" />
      <MessageComposer
        :reply-to="replyTo"
        :editing-message="editingMessage"
        @send="handleSend"
        @send-image="handleSendImage"
        @typing="handleTyping"
        @cancel-reply="setReplyTo(null)"
        @cancel-edit="cancelEditing"
        @save-edit="editMessage"
      />
    </div>
  </div>
</template>

<style scoped>
.combo-channel {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  position: relative;
}

.combo-channel.resizing {
  user-select: none;
}

.admin-overlay-wrap {
  position: absolute;
  top: 0.25rem;
  right: 0.25rem;
  z-index: 20;
}

.admin-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--text-muted);
  cursor: pointer;
  flex-shrink: 0;
  transition: all 0.15s;
}

.admin-toggle:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border-color: var(--text-muted);
}

.admin-panel-dropdown {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 0.25rem;
  width: min(380px, calc(100vw - 1rem));
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
}

.combo-video {
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.combo-video:not(.collapsed) {
  min-height: 120px;
  max-height: 80%;
}

.collapse-toggle {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  min-height: 48px;
  background: var(--bg-secondary);
  border: none;
  border-bottom: 1px solid var(--border);
  color: var(--text-secondary);
  font-size: 0.75rem;
  cursor: pointer;
  flex-shrink: 0;
}

.collapse-toggle:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.collapse-toggle svg {
  width: 14px;
  height: 14px;
}

.video-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}

.resize-handle {
  flex-shrink: 0;
  height: 6px;
  cursor: row-resize;
  background: var(--border);
  transition: background 0.15s;
  touch-action: none;
}

.resize-handle:hover {
  background: var(--accent);
}

.combo-chat {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

@media (max-width: 767px) {
  .combo-video:not(.collapsed) {
    min-height: 120px;
    max-height: 80%;
  }

  .collapse-toggle {
    min-height: 36px;
    padding: 0.35rem 0.75rem;
    font-size: 0.7rem;
  }
}

/* Desktop: side-by-side */
@media (min-width: 1024px) {
  .combo-channel {
    flex-direction: row;
  }

  .combo-video {
    border-right: 1px solid var(--border);
  }

  .combo-video:not(.collapsed) {
    min-height: unset;
    max-height: none;
    min-width: 20%;
    max-width: 80%;
  }

  .combo-chat {
    flex: 1;
  }

  .combo-video.collapsed {
    flex: 0 0 auto;
    border-right: 1px solid var(--border);
  }

  .resize-handle {
    width: 6px;
    height: auto;
    cursor: col-resize;
  }
}
</style>
