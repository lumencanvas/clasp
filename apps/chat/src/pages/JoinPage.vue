<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useClasp } from '../composables/useClasp.js'
import { useIdentity } from '../composables/useIdentity.js'
import { AVATAR_COLORS, USER_STATUSES, DEFAULT_RELAY_URL } from '../lib/constants.js'

const router = useRouter()
const { connecting, connected, error, url, connect } = useClasp()
const { displayName, avatarColor, status, setDisplayName, setAvatarColor, setStatus } = useIdentity()

const nameInput = ref(displayName.value)
const serverUrl = ref(url.value)

async function handleConnect() {
  if (!nameInput.value.trim()) return
  setDisplayName(nameInput.value.trim())
  url.value = serverUrl.value
  await connect(nameInput.value.trim())
  if (connected.value) {
    router.push('/chat')
  }
}
</script>

<template>
  <div class="join-page">
    <div class="join-card">
      <div class="join-header">
        <div class="logo">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"/>
          </svg>
        </div>
        <h1>CLASP Chat</h1>
        <p class="subtitle">Real-time messaging powered by the CLASP protocol</p>
      </div>

      <form class="join-form" @submit.prevent="handleConnect">
        <div class="field">
          <label>Display Name</label>
          <input
            v-model="nameInput"
            type="text"
            placeholder="Enter your name"
            autocomplete="off"
            maxlength="32"
            autofocus
          />
        </div>

        <div class="field">
          <label>Avatar Color</label>
          <div class="color-picker">
            <button
              v-for="color in AVATAR_COLORS"
              :key="color"
              type="button"
              :class="['color-swatch', { active: avatarColor === color }]"
              :style="{ background: color }"
              @click="setAvatarColor(color)"
            />
          </div>
        </div>

        <div class="field">
          <label>Status</label>
          <div class="status-picker">
            <button
              v-for="s in USER_STATUSES"
              :key="s.value"
              type="button"
              :class="['status-option', { active: status === s.value }]"
              @click="setStatus(s.value)"
            >
              <span class="status-swatch" :style="{ background: s.color }"></span>
              {{ s.label }}
            </button>
          </div>
        </div>

        <div class="field">
          <label>Server URL</label>
          <input
            v-model="serverUrl"
            type="text"
            placeholder="wss://relay.clasp.to"
          />
        </div>

        <button
          type="submit"
          class="connect-btn"
          :disabled="connecting || !nameInput.trim()"
        >
          <span v-if="connecting" class="spinner"></span>
          <span v-else>Connect</span>
        </button>

        <p v-if="error" class="error-text">{{ error }}</p>
      </form>
    </div>
  </div>
</template>

<style scoped>
.join-page {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  background: var(--bg-primary);
}

.join-card {
  width: 100%;
  max-width: 400px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 2rem;
}

.join-header {
  text-align: center;
  margin-bottom: 2rem;
}

.logo {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 56px;
  margin-bottom: 1rem;
}

.logo svg {
  width: 48px;
  height: 48px;
  color: var(--accent);
}

.join-header h1 {
  font-size: 1.5rem;
  letter-spacing: 0.12em;
  margin-bottom: 0.5rem;
}

.subtitle {
  font-size: 0.8rem;
  color: var(--text-secondary);
  line-height: 1.5;
}

.join-form {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.field label {
  font-size: 0.7rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--text-secondary);
}

.field input {
  padding: 0.75rem 1rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 0.9rem;
  transition: border-color 0.15s;
}

.field input:focus {
  outline: none;
  border-color: var(--accent);
}

.color-picker {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.color-swatch {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  transition: all 0.15s;
}

.color-swatch:hover {
  transform: scale(1.15);
}

.color-swatch.active {
  border-color: var(--text-primary);
  transform: scale(1.15);
}

.status-picker {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.status-option {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.4rem 0.75rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 4px;
  color: var(--text-secondary);
  font-size: 0.75rem;
  cursor: pointer;
  transition: all 0.15s;
}

.status-option:hover {
  background: var(--bg-active);
  color: var(--text-primary);
}

.status-option.active {
  border-color: var(--accent);
  color: var(--text-primary);
}

.status-swatch {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.connect-btn {
  min-height: 44px;
  padding: 0.75rem 1rem;
  background: var(--accent);
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 0.9rem;
  letter-spacing: 0.08em;
  transition: opacity 0.15s;
  margin-top: 0.5rem;
}

.connect-btn:hover:not(:disabled) {
  opacity: 0.9;
}

.connect-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.spinner {
  display: inline-block;
  width: 18px;
  height: 18px;
  border: 2px solid rgba(255,255,255,0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.error-text {
  color: var(--danger);
  font-size: 0.8rem;
  text-align: center;
}
</style>
