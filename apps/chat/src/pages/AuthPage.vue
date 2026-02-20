<script setup>
import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuth } from '../composables/useAuth.js'
import { useClasp } from '../composables/useClasp.js'
import { useIdentity } from '../composables/useIdentity.js'

const router = useRouter()
const route = useRoute()
const { authError, authLoading, authUserId, register, login } = useAuth()
const { connect, disconnect } = useClasp()
const { userId, setUserId, setDisplayName } = useIdentity()

// Clear any stale guest token so we get a fresh auth
disconnect()
localStorage.removeItem('clasp-chat-token')

const mode = ref('login') // 'login' | 'register'
const username = ref('')
const password = ref('')
const confirmPassword = ref('')

async function handleSubmit() {
  if (!username.value.trim() || !password.value) return

  if (mode.value === 'register') {
    if (password.value !== confirmPassword.value) {
      authError.value = 'Passwords do not match'
      return
    }
    if (password.value.length < 6) {
      authError.value = 'Password must be at least 6 characters'
      return
    }
    const ok = await register(username.value.trim(), password.value, userId.value)
    if (ok) {
      if (authUserId.value) setUserId(authUserId.value)
      setDisplayName(username.value.trim())
      await connect(username.value.trim())
      const joinParam = route.query.join
      router.push(joinParam ? { path: '/chat', query: { join: joinParam } } : '/chat')
    }
  } else {
    const ok = await login(username.value.trim(), password.value)
    if (ok) {
      if (authUserId.value) setUserId(authUserId.value)
      setDisplayName(username.value.trim())
      await connect(username.value.trim())
      const joinParam = route.query.join
      router.push(joinParam ? { path: '/chat', query: { join: joinParam } } : '/chat')
    }
  }
}
</script>

<template>
  <div class="auth-page">
    <div class="auth-card">
      <div class="auth-header">
        <div class="logo">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"/>
          </svg>
        </div>
        <h1>CLASP Chat</h1>
        <p class="subtitle">Secure, real-time messaging</p>
      </div>

      <div class="tab-toggle">
        <button
          :class="['tab', { active: mode === 'login' }]"
          @click="mode = 'login'; authError = null"
        >
          Login
        </button>
        <button
          :class="['tab', { active: mode === 'register' }]"
          @click="mode = 'register'; authError = null"
        >
          Register
        </button>
      </div>

      <form class="auth-form" @submit.prevent="handleSubmit">
        <div class="field">
          <label>Username</label>
          <input
            v-model="username"
            type="text"
            placeholder="Enter username"
            autocomplete="username"
            maxlength="32"
            autofocus
          />
        </div>

        <div class="field">
          <label>Password</label>
          <input
            v-model="password"
            type="password"
            placeholder="Enter password"
            autocomplete="current-password"
          />
        </div>

        <div v-if="mode === 'register'" class="field">
          <label>Confirm Password</label>
          <input
            v-model="confirmPassword"
            type="password"
            placeholder="Confirm password"
            autocomplete="new-password"
          />
        </div>

        <button
          type="submit"
          class="submit-btn"
          :disabled="authLoading || !username.trim() || !password"
        >
          <span v-if="authLoading" class="spinner"></span>
          <span v-else>{{ mode === 'login' ? 'Login' : 'Register' }}</span>
        </button>

        <p v-if="authError" class="error-text">{{ authError }}</p>

        <p class="alt-link">
          <template v-if="mode === 'login'">
            No account?
            <a href="#" @click.prevent="mode = 'register'; authError = null">Register</a>
          </template>
          <template v-else>
            Already have an account?
            <a href="#" @click.prevent="mode = 'login'; authError = null">Login</a>
          </template>
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
.auth-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  background: var(--bg-primary);
}

.auth-card {
  width: 100%;
  max-width: 400px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 2rem;
}

.auth-header {
  text-align: center;
  margin-bottom: 1.5rem;
}

.logo {
  display: inline-flex;
  width: 56px;
  height: 56px;
  margin-bottom: 1rem;
}

.logo svg {
  width: 48px;
  height: 48px;
  color: var(--accent);
}

.auth-header h1 {
  font-size: 1.5rem;
  letter-spacing: 0.12em;
  margin-bottom: 0.5rem;
}

.subtitle {
  font-size: 0.8rem;
  color: var(--text-secondary);
}

.tab-toggle {
  display: flex;
  gap: 0;
  margin-bottom: 1.5rem;
  border: 1px solid var(--border);
  border-radius: 4px;
  overflow: hidden;
}

.tab {
  flex: 1;
  padding: 0.6rem;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  font-size: 0.85rem;
  cursor: pointer;
  transition: all 0.15s;
}

.tab.active {
  background: var(--accent);
  color: white;
}

.auth-form {
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

.submit-btn {
  min-height: 44px;
  padding: 0.75rem 1rem;
  background: var(--accent);
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 0.9rem;
  letter-spacing: 0.08em;
  transition: opacity 0.15s;
  margin-top: 0.25rem;
}

.submit-btn:hover:not(:disabled) {
  opacity: 0.9;
}

.submit-btn:disabled {
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

.alt-link {
  font-size: 0.75rem;
  color: var(--text-muted);
  text-align: center;
}

.alt-link a {
  color: var(--accent2);
  text-decoration: none;
}

.alt-link a:hover {
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
}

.github-footer:hover {
  color: var(--text-secondary);
}

.github-footer svg {
  width: 16px;
  height: 16px;
}

@media (max-width: 480px) {
  .auth-page {
    padding: 1rem;
  }

  .auth-card {
    padding: 1.25rem;
  }

  .tab {
    min-height: 44px;
  }
}
</style>
