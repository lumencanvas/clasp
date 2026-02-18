<script setup>
import { ref, computed, toRef } from 'vue'
import { useVideoRoom } from '../composables/useVideoRoom.js'
import { useVideoLayout } from '../composables/useVideoLayout.js'
import { useIdentity } from '../composables/useIdentity.js'
import LocalPreview from './LocalPreview.vue'
import VideoGrid from './VideoGrid.vue'
import VideoControls from './VideoControls.vue'

const props = defineProps({
  roomId: { type: String, required: true },
})

const roomIdRef = toRef(props, 'roomId')
const { displayName, avatarColor } = useIdentity()

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
  getAudioOnly,
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

async function handleJoinCamera() {
  mediaLoading.value = true
  try {
    if (!localStream.value) await getUserMedia()
    await joinVideo()
  } finally {
    mediaLoading.value = false
  }
}

async function handleJoinAudio() {
  mediaLoading.value = true
  try {
    await getAudioOnly()
    await joinVideo()
  } finally {
    mediaLoading.value = false
  }
}

async function handleJoinSpectator() {
  await joinVideo()
}

function handleLeave() {
  leaveVideo()
  stopUserMedia()
}
</script>

<template>
  <div class="video-channel">
    <!-- Preview before joining -->
    <LocalPreview
      v-if="!inVideo"
      :stream="localStream"
      :loading="mediaLoading"
      :error="error"
      @join-camera="handleJoinCamera"
      @join-audio="handleJoinAudio"
      @join-spectator="handleJoinSpectator"
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
        @toggle-audio="toggleAudio"
        @toggle-video="toggleVideo"
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
</style>
