<script setup>
import { ref, watch, onMounted } from 'vue'
import UserAvatar from './UserAvatar.vue'

const props = defineProps({
  stream: { type: MediaStream, default: null },
  name: { type: String, default: '' },
  muted: { type: Boolean, default: false },
  audioEnabled: { type: Boolean, default: true },
  videoEnabled: { type: Boolean, default: true },
  isLocal: { type: Boolean, default: false },
  isScreenShare: { type: Boolean, default: false },
  avatarColor: { type: String, default: null },
  isPinned: { type: Boolean, default: false },
  isSpeaking: { type: Boolean, default: false },
})

const emit = defineEmits(['pin'])

const videoEl = ref(null)
let lastTap = 0
let isTouchDevice = false

function attachStream() {
  if (videoEl.value && props.stream) {
    videoEl.value.srcObject = props.stream
  }
}

function handleClick() {
  // On touch devices, only respond to double-tap (handled in touchend)
  if (isTouchDevice) return
  emit('pin')
}

function handleTouchStart() {
  isTouchDevice = true
}

function handleTouchEnd() {
  const now = Date.now()
  if (now - lastTap < 300) {
    emit('pin')
    lastTap = 0 // Reset to avoid triple-tap
  } else {
    lastTap = now
  }
}

watch(() => props.stream, attachStream)
onMounted(attachStream)
</script>

<template>
  <div
    :class="['video-tile', { local: isLocal, speaking: isSpeaking, pinned: isPinned }]"
    @click="handleClick"
    @touchstart.passive="handleTouchStart"
    @touchend.passive="handleTouchEnd"
  >
    <!-- Pin indicator -->
    <div v-if="isPinned" class="pin-indicator">
      <svg viewBox="0 0 24 24" fill="currentColor" width="12" height="12">
        <path d="M16 12V4h1V2H7v2h1v8l-2 2v2h5.2v6h1.6v-6H18v-2l-2-2z"/>
      </svg>
    </div>

    <video
      ref="videoEl"
      :class="{ mirrored: isLocal && !isScreenShare }"
      autoplay
      playsinline
      :muted="muted || isLocal"
    ></video>

    <div v-if="!videoEnabled || !stream" class="video-off-overlay">
      <UserAvatar :name="name" :color="avatarColor" :size="64" />
    </div>

    <div class="tile-overlay">
      <span class="tile-name">{{ name }}{{ isLocal ? ' (You)' : '' }}</span>
      <div class="tile-indicators">
        <svg v-if="!audioEnabled" class="indicator-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="1" y1="1" x2="23" y2="23"/>
          <path d="M9 9v3a3 3 0 0 0 5.12 2.12M15 9.34V4a3 3 0 0 0-5.94-.6"/>
          <path d="M17 16.95A7 7 0 0 1 5 12v-2m14 0v2c0 .76-.13 1.49-.35 2.17"/>
          <line x1="12" y1="19" x2="12" y2="23"/>
          <line x1="8" y1="23" x2="16" y2="23"/>
        </svg>
        <svg v-if="!videoEnabled" class="indicator-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M16 16v1a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2h2m5.66 0H14a2 2 0 0 1 2 2v3.34l1 1L23 7v10"/>
          <line x1="1" y1="1" x2="23" y2="23"/>
        </svg>
      </div>
    </div>
  </div>
</template>

<style scoped>
.video-tile {
  position: relative;
  background: var(--bg-tertiary);
  border-radius: 8px;
  overflow: hidden;
  aspect-ratio: 16/9;
  cursor: pointer;
  border: 2px solid transparent;
  transition: border-color 0.2s;
}

.video-tile.speaking {
  border-color: var(--success);
  box-shadow: 0 0 8px rgba(46, 204, 113, 0.4);
}

.video-tile.pinned {
  border-color: var(--accent);
}

.video-tile.speaking.pinned {
  border-color: var(--success);
}

.pin-indicator {
  position: absolute;
  top: 6px;
  left: 6px;
  z-index: 2;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  background: rgba(0, 0, 0, 0.6);
  border-radius: 4px;
  color: var(--accent);
}

.pin-indicator svg {
  width: 12px;
  height: 12px;
}

.video-tile video {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.video-tile video.mirrored {
  transform: scaleX(-1);
}

.video-off-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-tertiary);
}

.tile-overlay {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 0.5rem 0.75rem;
  background: linear-gradient(transparent, rgba(0,0,0,0.7));
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.tile-name {
  font-size: 0.75rem;
  color: white;
  text-shadow: 0 1px 2px rgba(0,0,0,0.5);
}

.tile-indicators {
  display: flex;
  gap: 0.35rem;
}

.indicator-icon {
  width: 14px;
  height: 14px;
  color: var(--danger);
}

@media (max-width: 480px) {
  .tile-name {
    font-size: 0.65rem;
  }

  .indicator-icon {
    width: 12px;
    height: 12px;
  }
}
</style>
