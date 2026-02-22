import { ref, computed } from 'vue'
import { AUTH_API_URL } from '../lib/constants.js'

const token = ref(localStorage.getItem('clasp-chat-token') || null)
const authUserId = ref(localStorage.getItem('clasp-chat-auth-userId') || null)
const authUsername = ref(localStorage.getItem('clasp-chat-auth-username') || null)
const authError = ref(null)
const authLoading = ref(false)

const isAuthenticated = computed(() => !!token.value)

async function register(username, password, existingUserId) {
  authLoading.value = true
  authError.value = null

  try {
    const body = { username, password }
    if (existingUserId) body.user_id = existingUserId
    const res = await fetch(`${AUTH_API_URL}/auth/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    })

    const data = await res.json()
    if (!res.ok) {
      authError.value = data.error || 'Registration failed'
      return false
    }

    token.value = data.token
    authUserId.value = data.user_id
    authUsername.value = data.username

    localStorage.setItem('clasp-chat-token', data.token)
    localStorage.setItem('clasp-chat-auth-userId', data.user_id)
    localStorage.setItem('clasp-chat-auth-username', data.username)

    return true
  } catch (e) {
    authError.value = e.message || 'Network error'
    return false
  } finally {
    authLoading.value = false
  }
}

async function login(username, password) {
  authLoading.value = true
  authError.value = null

  try {
    const res = await fetch(`${AUTH_API_URL}/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password }),
    })

    const data = await res.json()
    if (!res.ok) {
      authError.value = data.error || 'Login failed'
      return false
    }

    token.value = data.token
    authUserId.value = data.user_id
    authUsername.value = data.username

    localStorage.setItem('clasp-chat-token', data.token)
    localStorage.setItem('clasp-chat-auth-userId', data.user_id)
    localStorage.setItem('clasp-chat-auth-username', data.username)

    return true
  } catch (e) {
    authError.value = e.message || 'Network error'
    return false
  } finally {
    authLoading.value = false
  }
}

function logout() {
  token.value = null
  authUserId.value = null
  authUsername.value = null
  authError.value = null

  localStorage.removeItem('clasp-chat-token')
  localStorage.removeItem('clasp-chat-auth-userId')
  localStorage.removeItem('clasp-chat-auth-username')
  localStorage.removeItem('clasp-chat-userId')
}

export function useAuth() {
  return {
    token,
    authUserId,
    authUsername,
    authError,
    authLoading,
    isAuthenticated,
    register,
    login,
    logout,
  }
}
