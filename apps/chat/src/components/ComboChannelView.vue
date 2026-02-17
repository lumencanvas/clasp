<script setup>
import { ref, toRef } from 'vue'
import { useVideoRoom } from '../composables/useVideoRoom.js'
import { useChat } from '../composables/useChat.js'
import { useIdentity } from '../composables/useIdentity.js'
import LocalPreview from './LocalPreview.vue'
import VideoGrid from './VideoGrid.vue'
import VideoControls from './VideoControls.vue'
import MessageList from './MessageList.vue'
import MessageComposer from './MessageComposer.vue'
import TypingIndicator from './TypingIndicator.vue'

const props = defineProps({
  roomId: { type: String, required: true },
})

const roomIdRef = toRef(props, 'roomId')
const { displayName } = useIdentity()

// Chat composable
const {
  messages,
  typingList,
  sendMessage,
  handleTyping,
} = useChat(roomIdRef)

// Video composable
const {
  localStream,
  inVideo,
  audioEnabled,
  videoEnabled,
  error: videoError,
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
const videoCollapsed = ref(false)

async function handleGetMedia() {
  mediaLoading.value = true
  try { await getUserMedia() } finally { mediaLoading.value = false }
}

async function handleJoinVideo() {
  await joinVideo()
}

function handleLeaveVideo() {
  leaveVideo()
  stopUserMedia()
}
</script>

<template>
  <div class="combo-channel">
    <!-- Video section -->
    <div :class="['combo-video', { collapsed: videoCollapsed }]">
      <button class="collapse-toggle" @click="videoCollapsed = !videoCollapsed">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline v-if="videoCollapsed" points="6 9 12 15 18 9"/>
          <polyline v-else points="18 15 12 9 6 15"/>
        </svg>
        {{ videoCollapsed ? 'Show Video' : 'Hide Video' }}
      </button>

      <div v-show="!videoCollapsed" class="video-area">
        <LocalPreview
          v-if="!inVideo"
          :stream="localStream"
          :loading="mediaLoading"
          :error="videoError"
          @get-media="handleGetMedia"
          @join="handleJoinVideo"
        />
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
            @leave="handleLeaveVideo"
          />
        </template>
      </div>
    </div>

    <!-- Chat section -->
    <div class="combo-chat">
      <MessageList :messages="messages" />
      <TypingIndicator :users="typingList" />
      <MessageComposer
        @send="sendMessage"
        @typing="handleTyping"
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
}

.combo-video {
  display: flex;
  flex-direction: column;
  border-bottom: 1px solid var(--border);
}

.combo-video:not(.collapsed) {
  flex: 0 0 55%;
  min-height: 200px;
}

.collapse-toggle {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
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
}

.combo-chat {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

/* Desktop: side-by-side */
@media (min-width: 1024px) {
  .combo-channel {
    flex-direction: row;
  }

  .combo-video {
    border-bottom: none;
    border-right: 1px solid var(--border);
  }

  .combo-video:not(.collapsed) {
    flex: 0 0 60%;
    min-height: unset;
  }

  .combo-chat {
    flex: 1;
  }

  .collapse-toggle {
    display: none;
  }

  .combo-video.collapsed {
    display: none;
  }
}
</style>
