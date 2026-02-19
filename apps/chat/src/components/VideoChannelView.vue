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

const { layout, spotlightPeer, setLayout } = useVideoLayout(isScreenSharing)

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
</script>

<template>
  <div class="video-channel">
    <div v-if="isAdmin" class="video-header-bar">
      <button
        class="admin-toggle"
        aria-label="Room settings"
        title="Room settings"
        @click="showAdmin = !showAdmin"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
          <circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
        </svg>
      </button>
    </div>
    <AdminPanel
      v-if="showAdmin"
      :room-id="roomId"
      :members="sortedParticipants"
      @close="showAdmin = false"
      @delete-room="emit('delete-room', $event)"
    />

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
      />
      <VideoControls
        :audio-enabled="audioEnabled"
        :video-enabled="videoEnabled"
        :is-screen-sharing="isScreenSharing"
        :layout="layout"
        @toggle-audio="handleToggleAudio"
        @toggle-video="handleToggleVideo"
        @share-screen="shareScreen"
        @set-layout="setLayout"
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
}

.video-header-bar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  flex-shrink: 0;
}

.admin-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--text-muted);
  cursor: pointer;
  flex-shrink: 0;
  margin: 0.25rem 0.5rem;
}

.admin-toggle:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}
</style>
