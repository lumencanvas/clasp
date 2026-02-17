<script setup>
import { ref, toRef } from 'vue'
import { useVideoRoom } from '../composables/useVideoRoom.js'
import { useIdentity } from '../composables/useIdentity.js'
import LocalPreview from './LocalPreview.vue'
import VideoGrid from './VideoGrid.vue'
import VideoControls from './VideoControls.vue'

const props = defineProps({
  roomId: { type: String, required: true },
})

const roomIdRef = toRef(props, 'roomId')
const { displayName } = useIdentity()

const {
  localStream,
  inVideo,
  audioEnabled,
  videoEnabled,
  error,
  peerList,
  getUserMedia,
  joinVideo,
  leaveVideo,
  toggleAudio,
  toggleVideo,
  shareScreen,
  stopUserMedia,
} = useVideoRoom(roomIdRef)

const mediaLoading = ref(false)

async function handleGetMedia() {
  mediaLoading.value = true
  try {
    await getUserMedia()
  } finally {
    mediaLoading.value = false
  }
}

async function handleJoin() {
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
      @get-media="handleGetMedia"
      @join="handleJoin"
    />

    <!-- Active video session -->
    <template v-else>
      <VideoGrid
        :local-stream="localStream"
        :local-name="displayName"
        :audio-enabled="audioEnabled"
        :video-enabled="videoEnabled"
        :peers="peerList"
      />
      <VideoControls
        :audio-enabled="audioEnabled"
        :video-enabled="videoEnabled"
        @toggle-audio="toggleAudio"
        @toggle-video="toggleVideo"
        @share-screen="shareScreen"
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
