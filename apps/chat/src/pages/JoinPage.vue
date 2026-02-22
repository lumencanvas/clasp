<script setup>
import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useClasp } from '../composables/useClasp.js'
import { useIdentity } from '../composables/useIdentity.js'
import { AVATAR_COLORS, USER_STATUSES, DEFAULT_RELAY_URL, AUTH_API_URL } from '../lib/constants.js'

const router = useRouter()
const route = useRoute()
const { connecting, connected, error: claspError, url, connect } = useClasp()
const { userId, displayName, avatarColor, status, setDisplayName, setAvatarColor, setStatus, setUserId } = useIdentity()

const nameInput = ref(displayName.value)
const serverUrl = ref(url.value)
const localError = ref(null)
const showAdvanced = ref(false)

async function handleConnect() {
  if (!nameInput.value.trim()) return
  localError.value = null
  setDisplayName(nameInput.value.trim())
  url.value = serverUrl.value

  // Try guest token for auth-enabled relays
  try {
    const res = await fetch(`${AUTH_API_URL}/auth/guest`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: nameInput.value.trim(), user_id: userId.value }),
    })
    if (res.ok) {
      const data = await res.json()
      localStorage.setItem('clasp-chat-token', data.token)
    } else if (res.status === 409) {
      const data = await res.json()
      const msg = data.error || ''
      if (msg.includes('identity belongs to a registered user')) {
        // Stale UUID from a previous registration — generate a fresh one and retry once
        setUserId(crypto.randomUUID())
        const retry = await fetch(`${AUTH_API_URL}/auth/guest`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ name: nameInput.value.trim(), user_id: userId.value }),
        })
        if (retry.ok) {
          const retryData = await retry.json()
          localStorage.setItem('clasp-chat-token', retryData.token)
        } else {
          const retryData = await retry.json()
          localError.value = retryData.error || 'That name is taken by a registered user'
          return
        }
      } else {
        localError.value = msg || 'That name is taken by a registered user'
        return
      }
    }
  } catch {
    // Open relay, no auth needed
  }

  await connect(nameInput.value.trim())
  if (connected.value) {
    const joinParam = route.query.join
    router.push(joinParam ? { path: '/chat', query: { join: joinParam } } : '/chat')
  }
}
</script>

<template>
  <div class="join-page">
    <div class="join-card">
      <div class="join-header">
        <img src="/logo.svg" alt="CLASP" class="header-logo" />
        <h1 class="header-title"><span class="title-clasp">CLASP</span><span class="title-dot">.chat</span></h1>
        <p class="subtitle">Real-time messaging powered by the <a href="https://clasp.to" target="_blank" rel="noopener noreferrer">CLASP protocol</a></p>
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
              :style="{ '--swatch-color': color }"
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

        <button
          type="button"
          class="advanced-toggle"
          @click="showAdvanced = !showAdvanced"
        >
          <svg :class="['advanced-chevron', { open: showAdvanced }]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
            <polyline points="9 18 15 12 9 6"/>
          </svg>
          Advanced
        </button>

        <div v-if="showAdvanced" class="field">
          <label>Server URL</label>
          <input
            v-model="serverUrl"
            type="text"
            placeholder="wss://relay.clasp.chat"
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

        <p v-if="localError || claspError" class="error-text">{{ localError || claspError }}</p>

        <p class="alt-action">
          Want a persistent account? <router-link :to="route.query.join ? { path: '/auth', query: { join: route.query.join } } : '/auth'">Sign up or sign in</router-link>
        </p>
      </form>
    </div>

    <a class="github-footer" href="https://github.com/lumencanvas/clasp" target="_blank" rel="noopener noreferrer">
      <svg viewBox="0 0 24 24" fill="currentColor" width="16" height="16">
        <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"/>
      </svg>
      View on GitHub
    </a>
  </div>
</template>

<style scoped>
.join-page {
  min-height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  background: var(--bg-primary);
  overflow-y: auto;
}

.join-card {
  width: 100%;
  max-width: 420px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 2rem 2rem;
}

.join-header {
  text-align: center;
  margin-bottom: 1.5rem;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.header-logo {
  width: 80px;
  height: 80px;
  margin-bottom: 0.5rem;
}

.header-title {
  font-family: 'Oswald', 'Arial Narrow', sans-serif;
  font-size: 2rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  margin-bottom: 0.25rem;
}

.title-clasp {
  color: var(--text-primary);
}

.title-dot {
  color: var(--accent);
}

.subtitle {
  font-size: 0.8rem;
  color: var(--text-muted);
  line-height: 1.4;
}

.subtitle a {
  color: var(--accent);
  text-decoration: none;
}

.subtitle a:hover {
  text-decoration: underline;
}

.join-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.field label {
  font-size: 0.7rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--text-secondary);
}

.field input {
  padding: 0.65rem 1rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 0.9rem;
  transition: border-color 0.15s;
}

.field input:focus {
  outline: none;
  border-color: var(--accent);
}

.color-picker {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  justify-content: center;
}

.color-swatch {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  transition: all 0.15s;
  background: var(--swatch-color);
}

.color-swatch:hover {
  transform: scale(1.15);
  border-color: rgba(255,255,255,0.25);
}

.color-swatch.active {
  border-color: var(--text-primary);
  transform: scale(1.15);
  box-shadow: 0 0 0 2px var(--bg-secondary), 0 0 0 3.5px var(--swatch-color);
}

.status-picker {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.35rem;
}

.status-option {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.4rem;
  padding: 0.4rem 0.5rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-secondary);
  font-size: 0.75rem;
  cursor: pointer;
  transition: all 0.15s;
}

.status-option:hover {
  background: var(--bg-active);
  color: var(--text-primary);
  border-color: var(--text-muted);
}

.status-option.active {
  border-color: var(--accent);
  color: var(--text-primary);
  background: rgba(230,57,70,0.06);
}

.status-swatch {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.advanced-toggle {
  display: flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0;
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 0.75rem;
  cursor: pointer;
  align-self: flex-start;
  transition: color 0.15s;
}

.advanced-toggle:hover {
  color: var(--text-secondary);
}

.advanced-chevron {
  transition: transform 0.2s;
}

.advanced-chevron.open {
  transform: rotate(90deg);
}

.connect-btn {
  min-height: 44px;
  padding: 0.7rem 1rem;
  background: var(--accent);
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 0.9rem;
  font-weight: 600;
  letter-spacing: 0.06em;
  transition: all 0.15s;
}

.connect-btn:hover:not(:disabled) {
  filter: brightness(1.1);
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

.alt-action {
  font-size: 0.8rem;
  color: var(--text-muted);
  text-align: center;
}

.alt-action a {
  color: var(--accent);
  text-decoration: none;
}

.alt-action a:hover {
  text-decoration: underline;
}

.github-footer {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.4rem;
  margin-top: 1rem;
  font-size: 0.75rem;
  color: var(--text-muted);
  text-decoration: none;
  transition: color 0.15s;
  flex-shrink: 0;
}

.github-footer:hover {
  color: var(--text-secondary);
}

.github-footer svg {
  width: 16px;
  height: 16px;
}

@media (max-width: 480px) {
  .join-page {
    padding: 0.75rem;
    justify-content: flex-start;
    padding-top: 1.5rem;
  }

  .join-card {
    padding: 1.25rem 1rem;
  }

  .join-header {
    margin-bottom: 1rem;
  }

  .header-logo {
    width: 56px;
    height: 56px;
    margin-bottom: 0.4rem;
  }

  .header-title {
    font-size: 1.6rem;
    margin-bottom: 0.15rem;
  }

  .join-form {
    gap: 0.75rem;
  }

  .color-swatch {
    width: 28px;
    height: 28px;
  }

  .color-picker {
    gap: 0.4rem;
  }

  .status-option {
    padding: 0.35rem 0.4rem;
    font-size: 0.7rem;
  }

  .github-footer {
    margin-top: 0.75rem;
  }
}
</style>
