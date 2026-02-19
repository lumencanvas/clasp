<script setup>
import { ref, watch, nextTick } from 'vue'

const props = defineProps({
  stream: { type: MediaStream, default: null },
  loading: { type: Boolean, default: false },
  error: { type: String, default: null },
})

const emit = defineEmits(['join', 'request-camera', 'stop-camera'])

const videoEl = ref(null)
const audioOn = ref(false)
const videoOn = ref(false)

async function attachStream() {
  // Wait for Vue to render the <video> element after v-if becomes true
  await nextTick()
  if (videoEl.value && props.stream) {
    videoEl.value.srcObject = props.stream
  }
}

watch(() => props.stream, attachStream)

function toggleMic() {
  audioOn.value = !audioOn.value
}

function toggleCamera() {
  videoOn.value = !videoOn.value
  if (videoOn.value) {
    emit('request-camera')
  } else {
    emit('stop-camera')
  }
}

function handleJoin() {
  emit('join', { audio: audioOn.value, video: videoOn.value })
}
</script>

<template>
  <div class="local-preview">
    <div class="preview-container">
      <video
        v-if="stream && videoOn"
        ref="videoEl"
        autoplay
        playsinline
        muted
      ></video>
      <div v-else class="preview-placeholder">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <polygon points="23 7 16 12 23 17 23 7"/>
          <rect x="1" y="5" width="15" height="14" rx="2" ry="2"/>
        </svg>
        <p>{{ videoOn && !stream ? 'Starting camera...' : 'Camera off' }}</p>
      </div>
    </div>

    <p v-if="error" class="error-text">{{ error }}</p>

    <div class="toggle-row">
      <button
        :class="['toggle-btn', { off: !audioOn }]"
        @click="toggleMic"
        :title="audioOn ? 'Mute mic' : 'Unmute mic'"
      >
        <!-- Mic on -->
        <svg v-if="audioOn" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"/>
          <path d="M19 10v2a7 7 0 0 1-14 0v-2"/>
          <line x1="12" y1="19" x2="12" y2="23"/>
          <line x1="8" y1="23" x2="16" y2="23"/>
        </svg>
        <!-- Mic off -->
        <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="1" y1="1" x2="23" y2="23"/>
          <path d="M9 9v3a3 3 0 0 0 5.12 2.12M15 9.34V4a3 3 0 0 0-5.94-.6"/>
          <path d="M17 16.95A7 7 0 0 1 5 12v-2m14 0v2c0 .67-.1 1.32-.27 1.93"/>
          <line x1="12" y1="19" x2="12" y2="23"/>
          <line x1="8" y1="23" x2="16" y2="23"/>
        </svg>
      </button>
      <button
        :class="['toggle-btn', { off: !videoOn }]"
        @click="toggleCamera"
        :title="videoOn ? 'Turn off camera' : 'Turn on camera'"
      >
        <!-- Camera on -->
        <svg v-if="videoOn" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polygon points="23 7 16 12 23 17 23 7"/>
          <rect x="1" y="5" width="15" height="14" rx="2" ry="2"/>
        </svg>
        <!-- Camera off -->
        <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M16 16v1a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2h2m5.66 0H14a2 2 0 0 1 2 2v3.34l1 1L23 7v10"/>
          <line x1="1" y1="1" x2="23" y2="23"/>
        </svg>
      </button>
    </div>

    <button
      class="join-btn"
      @click="handleJoin"
      :disabled="loading"
    >
      <span v-if="loading" class="spinner"></span>
      <template v-else>Join Call</template>
    </button>
  </div>
</template>

<style scoped>
.local-preview {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  padding: 2rem;
  height: 100%;
}

.preview-container {
  width: 100%;
  max-width: 480px;
  aspect-ratio: 16/9;
  background: var(--bg-tertiary);
  border-radius: 8px;
  overflow: hidden;
}

.preview-container video {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transform: scaleX(-1);
}

.preview-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
  color: var(--text-muted);
}

.preview-placeholder svg {
  width: 48px;
  height: 48px;
}

.preview-placeholder p {
  font-size: 0.85rem;
}

.error-text {
  color: var(--danger);
  font-size: 0.8rem;
}

.toggle-row {
  display: flex;
  gap: 0.75rem;
}

.toggle-btn {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  border: none;
  background: var(--bg-tertiary);
  color: var(--text-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.15s;
}

.toggle-btn svg {
  width: 20px;
  height: 20px;
}

.toggle-btn:hover {
  background: var(--bg-active);
}

.toggle-btn.off {
  background: var(--danger);
  color: white;
}

.toggle-btn.off:hover {
  opacity: 0.9;
}

.join-btn {
  min-height: 48px;
  padding: 0.75rem 2rem;
  background: var(--success);
  border: none;
  border-radius: 4px;
  color: white;
  font-size: 0.9rem;
  font-weight: 600;
  letter-spacing: 0.04em;
  cursor: pointer;
  transition: opacity 0.15s;
}

.join-btn:hover {
  opacity: 0.9;
}

.join-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.spinner {
  display: inline-block;
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255,255,255,0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

@media (max-width: 767px) {
  .local-preview {
    padding: 0.75rem;
  }
}
</style>
