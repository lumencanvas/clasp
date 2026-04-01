import { ref, shallowRef, readonly } from 'vue'

const RELAY_URL = import.meta.env.VITE_RELAY_URL || 'wss://demo-relay.clasp.to'
const AUTH_URL = import.meta.env.VITE_AUTH_URL || 'https://demo-relay.clasp.to'

const client = shallowRef(null)
const connected = ref(false)
const sessionId = ref(null)
const userName = ref(localStorage.getItem('clasp_demo_name') || '')
const authToken = ref(localStorage.getItem('clasp_demo_token') || '')
const authMode = ref(localStorage.getItem('clasp_demo_auth') || '') // 'guest' | 'user'
const error = ref(null)

let activeToken = null

async function authRequest(endpoint, body) {
  const res = await fetch(`${AUTH_URL}/auth/${endpoint}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || `Auth ${endpoint} failed`)
  }
  return res.json()
}

function saveAuth(token, name, mode) {
  authToken.value = token
  userName.value = name
  authMode.value = mode
  localStorage.setItem('clasp_demo_token', token)
  localStorage.setItem('clasp_demo_name', name)
  localStorage.setItem('clasp_demo_auth', mode)
}

async function loginAsGuest(name) {
  const data = await authRequest('guest', { name })
  saveAuth(data.token, name, 'guest')
  sessionId.value = data.session_id || data.sessionId || null
  return data
}

async function register(username, password) {
  const data = await authRequest('register', { username, password })
  saveAuth(data.token, username, 'user')
  sessionId.value = data.session_id || data.sessionId || null
  return data
}

async function login(username, password) {
  const data = await authRequest('login', { username, password })
  saveAuth(data.token, username, 'user')
  sessionId.value = data.session_id || data.sessionId || null
  return data
}

function logout() {
  disconnect()
  authToken.value = ''
  sessionId.value = null
  userName.value = ''
  authMode.value = ''
  activeToken = null
  localStorage.removeItem('clasp_demo_token')
  localStorage.removeItem('clasp_demo_name')
  localStorage.removeItem('clasp_demo_auth')
}

function disconnect() {
  if (client.value) {
    try { client.value.close() } catch {}
    client.value = null
  }
  connected.value = false
  activeToken = null
}

/**
 * Ensure we have a valid auth token. Guest tokens are refreshed every time
 * to avoid the 10-second timeout when a stale token hits the relay.
 * Registered user tokens are reused (they can re-login if expired).
 */
async function ensureAuth(fallbackName) {
  // Always get a fresh guest token to avoid stale token issues.
  const name = userName.value || fallbackName || 'guest'
  try {
    await loginAsGuest(name)
  } catch {
    // Name might conflict with a registered user -- retry with a generic name
    await loginAsGuest('guest-' + Date.now().toString(36).slice(-6))
  }
}

/**
 * Connect to the relay. Reuses existing connection if the token hasn't changed.
 * Call ensureAuth() before this to guarantee a valid token.
 */
async function connect() {
  const token = authToken.value

  // Reuse existing live connection opened with same token
  if (client.value && activeToken === token) {
    return client.value
  }

  // Close stale connection
  if (client.value) {
    try { client.value.close() } catch {}
    client.value = null
    connected.value = false
  }

  const { ClaspBuilder } = await import('@clasp-to/core')
  const builder = new ClaspBuilder(RELAY_URL)
    .withName(userName.value || 'demo-user')
    .withReconnect(true)
  if (token) builder.withToken(token)

  console.log('[relay] connecting to', RELAY_URL, 'with token:', token?.slice(0, 15))
  const c = await builder.connect()
  console.log('[relay] connected! session:', c.session)

  client.value = c
  connected.value = true
  activeToken = token
  sessionId.value = c.session || null
  error.value = null

  c.onDisconnect(() => { connected.value = false })
  c.onError((e) => { error.value = e?.message || 'Connection error' })
  c.onReconnect(() => { connected.value = true; error.value = null })

  return c
}

export function useRelay() {
  return {
    client: readonly(client),
    connected: readonly(connected),
    sessionId: readonly(sessionId),
    userName,
    authToken: readonly(authToken),
    authMode: readonly(authMode),
    error: readonly(error),
    RELAY_URL,
    AUTH_URL,
    connect,
    disconnect,
    ensureAuth,
    loginAsGuest,
    register,
    login,
    logout,
  }
}
