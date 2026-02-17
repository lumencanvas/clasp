<script setup>
import { ref, watch, onMounted } from 'vue'

const props = defineProps({
  stream: { type: MediaStream, default: null },
  loading: { type: Boolean, default: false },
  error: { type: String, default: null },
})

const emit = defineEmits(['join', 'get-media'])

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
      <button
        v-if="!stream"
        class="preview-btn"
        @click="emit('get-media')"
        :disabled="loading"
      >
        <span v-if="loading" class="spinner"></span>
        <span v-else>Enable Camera</span>
      </button>
      <button
        v-else
        class="preview-btn join"
        @click="emit('join')"
      >
        Join Video
      </button>
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
}

.preview-btn {
  min-height: 44px;
  padding: 0.75rem 1.5rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 4px;
  color: var(--text-primary);
  font-size: 0.85rem;
  transition: all 0.15s;
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
