<script setup>
import { ref, watch, onMounted } from 'vue'

const props = defineProps({
  stream: { type: MediaStream, default: null },
  name: { type: String, default: '' },
  muted: { type: Boolean, default: false },
  audioEnabled: { type: Boolean, default: true },
  videoEnabled: { type: Boolean, default: true },
  isLocal: { type: Boolean, default: false },
})

const videoEl = ref(null)

function attachStream() {
  if (videoEl.value && props.stream) {
    videoEl.value.srcObject = props.stream
  }
}

watch(() => props.stream, attachStream)
onMounted(attachStream)
</script>

<template>
  <div :class="['video-tile', { local: isLocal }]">
    <video
      ref="videoEl"
      autoplay
      playsinline
      :muted="muted || isLocal"
    ></video>

    <div v-if="!videoEnabled" class="video-off-overlay">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4 12.5-12.5z"/>
      </svg>
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
          <path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4 12.5-12.5z"/>
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
}

.video-tile video {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.video-tile.local video {
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

.video-off-overlay svg {
  width: 48px;
  height: 48px;
  color: var(--text-muted);
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
</style>
