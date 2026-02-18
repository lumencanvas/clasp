<script setup>
import { ref, watch, onMounted } from 'vue'

const props = defineProps({
  stream: { type: MediaStream, default: null },
  loading: { type: Boolean, default: false },
  error: { type: String, default: null },
})

const emit = defineEmits(['join-camera', 'join-audio', 'join-spectator'])

const videoEl = ref(null)
const cameraRequested = ref(false)

function attachStream() {
  if (videoEl.value && props.stream) {
    videoEl.value.srcObject = props.stream
  }
}

watch(() => props.stream, (s) => {
  attachStream()
  if (s) cameraRequested.value = true
})
onMounted(attachStream)
</script>

<template>
  <div class="local-preview">
    <div class="preview-container">
      <video
        v-if="stream"
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
        <p>Camera preview</p>
      </div>
    </div>

    <p v-if="error" class="error-text">{{ error }}</p>

    <div class="preview-actions">
      <!-- If camera stream is already active, show simple Join button -->
      <template v-if="stream">
        <button class="preview-btn join" @click="emit('join-camera')">
          Join Video
        </button>
      </template>

      <!-- Otherwise show three options -->
      <template v-else>
        <button
          class="preview-btn join"
          @click="emit('join-camera')"
          :disabled="loading"
        >
          <span v-if="loading" class="spinner"></span>
          <template v-else>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polygon points="23 7 16 12 23 17 23 7"/>
              <rect x="1" y="5" width="15" height="14" rx="2" ry="2"/>
            </svg>
            Join with Camera
          </template>
        </button>
        <button
          class="preview-btn audio-btn"
          @click="emit('join-audio')"
          :disabled="loading"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"/>
            <path d="M19 10v2a7 7 0 0 1-14 0v-2"/>
            <line x1="12" y1="19" x2="12" y2="23"/>
            <line x1="8" y1="23" x2="16" y2="23"/>
          </svg>
          Join with Audio
        </button>
        <button
          class="preview-btn"
          @click="emit('join-spectator')"
          :disabled="loading"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
            <circle cx="12" cy="12" r="3"/>
          </svg>
          Spectate
        </button>
      </template>
    </div>
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

.preview-actions {
  display: flex;
  gap: 0.75rem;
  flex-wrap: wrap;
  justify-content: center;
}

.preview-btn {
  min-height: 44px;
  padding: 0.75rem 1.25rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 4px;
  color: var(--text-primary);
  font-size: 0.85rem;
  transition: all 0.15s;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.preview-btn svg {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.preview-btn:hover {
  background: var(--bg-active);
}

.preview-btn.join {
  background: var(--success);
  border-color: var(--success);
  color: white;
}

.preview-btn.join:hover {
  opacity: 0.9;
}

.preview-btn.audio-btn {
  background: var(--accent);
  border-color: var(--accent);
  color: white;
}

.preview-btn.audio-btn:hover {
  opacity: 0.9;
}

.preview-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.spinner {
  display: inline-block;
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255,255,255,0.3);
  border-top-color: var(--text-primary);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
