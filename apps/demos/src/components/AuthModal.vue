<script setup>
import { ref } from 'vue'
import { useRelay } from '../composables/useRelay.js'

const emit = defineEmits(['close'])
const { loginAsGuest, register, login, connect } = useRelay()

const mode = ref('guest') // 'guest' | 'login' | 'register'
const name = ref('')
const username = ref('')
const password = ref('')
const loading = ref(false)
const err = ref('')

async function submit() {
  err.value = ''
  loading.value = true
  try {
    if (mode.value === 'guest') {
      await loginAsGuest(name.value || 'Guest')
    } else if (mode.value === 'register') {
      await register(username.value, password.value)
    } else {
      await login(username.value, password.value)
    }
    await connect()
    emit('close')
  } catch (e) {
    err.value = e.message || 'Something went wrong'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="modal fade-in">
      <div class="modal-head">
        <span class="modal-title">Connect to CLASP</span>
        <button class="modal-close" @click="emit('close')">&times;</button>
      </div>

      <div class="tabs">
        <button :class="{ active: mode === 'guest' }" @click="mode = 'guest'">Guest</button>
        <button :class="{ active: mode === 'login' }" @click="mode = 'login'">Login</button>
        <button :class="{ active: mode === 'register' }" @click="mode = 'register'">Register</button>
      </div>

      <form @submit.prevent="submit" class="modal-body">
        <template v-if="mode === 'guest'">
          <label>
            <span class="lbl">Display name</span>
            <input v-model="name" placeholder="Anonymous" autocomplete="off" />
          </label>
          <p class="hint">No account needed. Jump right in.</p>
        </template>

        <template v-else>
          <label>
            <span class="lbl">Username</span>
            <input v-model="username" placeholder="username" autocomplete="username" required />
          </label>
          <label>
            <span class="lbl">Password</span>
            <input v-model="password" type="password" placeholder="password" autocomplete="current-password" required />
          </label>
          <p v-if="mode === 'register'" class="hint">
            Create an account to persist your identity across sessions.
          </p>
        </template>

        <p v-if="err" class="err">{{ err }}</p>

        <button type="submit" class="btn-primary" :disabled="loading">
          {{ loading ? 'Connecting...' : mode === 'guest' ? 'Join as Guest' : mode === 'login' ? 'Login' : 'Create Account' }}
        </button>
      </form>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  background: rgba(0,0,0,0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
}
.modal {
  background: var(--card);
  border: 1px solid var(--bdr2);
  border-radius: 6px;
  width: 100%;
  max-width: 380px;
  overflow: hidden;
}
.modal-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  border-bottom: 1px solid var(--bdr);
}
.modal-title {
  font-family: var(--head);
  font-size: 12px;
  letter-spacing: 0.15em;
  text-transform: uppercase;
  color: var(--br);
}
.modal-close {
  font-size: 20px;
  color: var(--dim);
  line-height: 1;
}
.modal-close:hover { color: var(--br); }
.tabs {
  display: flex;
  border-bottom: 1px solid var(--bdr);
}
.tabs button {
  flex: 1;
  padding: 10px;
  font-size: 11px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--dim);
  border-bottom: 2px solid transparent;
  transition: color 0.15s, border-color 0.15s;
}
.tabs button.active {
  color: var(--teal);
  border-bottom-color: var(--teal);
}
.modal-body {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
label { display: flex; flex-direction: column; gap: 4px; }
.lbl {
  font-size: 10px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--dim);
}
.hint {
  font-size: 11px;
  color: var(--dim);
  line-height: 1.5;
}
.err {
  font-size: 11px;
  color: var(--red);
  background: var(--red-d);
  padding: 8px 10px;
  border-radius: var(--r);
}
.btn-primary {
  width: 100%;
  padding: 10px;
  background: var(--teal);
  color: var(--bg);
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  border-radius: var(--r);
  transition: opacity 0.15s;
}
.btn-primary:hover { opacity: 0.9; }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
