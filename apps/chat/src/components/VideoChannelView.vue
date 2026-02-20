<script setup>
import { ref, computed, toRef, onUnmounted } from 'vue'
import { useVideoRoom } from '../composables/useVideoRoom.js'
import { useVideoLayout } from '../composables/useVideoLayout.js'
import { useIdentity } from '../composables/useIdentity.js'
import { useAdmin } from '../composables/useAdmin.js'
import LocalPreview from './LocalPreview.vue'
import VideoGrid from './VideoGrid.vue'
import VideoControls from './VideoControls.vue'
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

const {
  localStream,
  inVideo,
  audioEnabled,
  videoEnabled,
  isScreenSharing,
  error,
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

const sortedParticipants = computed(() => {
  return participantList.value
    .map(p => ({ id: p.id, name: p.name, avatarColor: p.avatarColor }))
    .sort((a, b) => a.name.localeCompare(b.name))
})

const onlineCount = computed(() => participantList.value.length + (inVideo.value ? 1 : 0))

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

function handleLeave() {
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
  <div class="video-channel">
    <!-- Admin overlay (positioned absolutely so it doesn't break layout) -->
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

    <!-- Preview before joining -->
    <LocalPreview
      v-if="!inVideo"
      :stream="localStream"
      :loading="mediaLoading"
      :error="error"
      @join="handleJoin"
      @request-camera="requestCameraPreview"
      @stop-camera="stopCameraPreview"
    />

    <!-- Active video session -->
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
        @leave="handleLeave"
      />
    </template>
  </div>
</template>

<style scoped>
.video-channel {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  background: var(--bg-primary);
  position: relative;
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
  width: 36px;
  height: 36px;
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
  width: min(380px, 90vw);
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
}
</style>
