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

// Track which token the current WebSocket was opened with
let activeToken = null
let reconnectTimer = null

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
  clearTimeout(reconnectTimer)
  if (client.value) {
    try { client.value.close() } catch {}
    client.value = null
  }
  connected.value = false
  activeToken = null
}

/**
 * Connect to the relay. Reuses the existing connection if the token hasn't
 * changed. If the token changed (user signed in/out), disconnects the old
 * session and opens a new one. If the stored token is stale (relay restarted),
 * falls back to a fresh guest login automatically.
 */
async function connect() {
  const token = authToken.value

  // Reuse existing connection if it was opened with the same token
  if (client.value && activeToken === token) {
    return client.value
  }

  // Token changed or no client -- close old connection if any
  if (client.value) {
    try { client.value.close() } catch {}
    client.value = null
    connected.value = false
  }

  const { ClaspBuilder } = await import('@clasp-to/core')

  async function tryConnect(t) {
    const builder = new ClaspBuilder(RELAY_URL)
      .withName(userName.value || 'demo-user')
      .withReconnect(true)
    if (t) builder.withToken(t)
    return builder.connect()
  }

  let c
  try {
    c = await tryConnect(token)
  } catch {
    // Token may be stale (relay restarted, token expired). Get a fresh guest token.
    if (token) {
      const name = userName.value || 'guest'
      await loginAsGuest(name)
      c = await tryConnect(authToken.value)
    } else {
      throw new Error('Connection failed')
    }
  }

  client.value = c
  connected.value = true
  activeToken = authToken.value
  sessionId.value = c.session || null
  error.value = null

  c.onDisconnect(() => {
    connected.value = false
  })

  c.onError((e) => {
    error.value = e?.message || 'Connection error'
  })

  c.onReconnect(() => {
    connected.value = true
    error.value = null
  })

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
    loginAsGuest,
    register,
    login,
    logout,
  }
}
