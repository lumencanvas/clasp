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

let reconnectTimer = null
let heartbeatTimer = null

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

async function loginAsGuest(name) {
  const data = await authRequest('guest', { name })
  authToken.value = data.token
  sessionId.value = data.session_id || data.sessionId || null
  userName.value = name
  authMode.value = 'guest'
  localStorage.setItem('clasp_demo_token', data.token)
  localStorage.setItem('clasp_demo_name', name)
  localStorage.setItem('clasp_demo_auth', 'guest')
  return data
}

async function register(username, password) {
  const data = await authRequest('register', { username, password })
  authToken.value = data.token
  sessionId.value = data.session_id || data.sessionId || null
  userName.value = username
  authMode.value = 'user'
  localStorage.setItem('clasp_demo_token', data.token)
  localStorage.setItem('clasp_demo_name', username)
  localStorage.setItem('clasp_demo_auth', 'user')
  return data
}

async function login(username, password) {
  const data = await authRequest('login', { username, password })
  authToken.value = data.token
  sessionId.value = data.session_id || data.sessionId || null
  userName.value = username
  authMode.value = 'user'
  localStorage.setItem('clasp_demo_token', data.token)
  localStorage.setItem('clasp_demo_name', username)
  localStorage.setItem('clasp_demo_auth', 'user')
  return data
}

function logout() {
  disconnect()
  authToken.value = ''
  sessionId.value = null
  userName.value = ''
  authMode.value = ''
  localStorage.removeItem('clasp_demo_token')
  localStorage.removeItem('clasp_demo_name')
  localStorage.removeItem('clasp_demo_auth')
}

async function connect() {
  if (client.value) return client.value

  const { ClaspBuilder } = await import('@clasp-to/core')

  const builder = new ClaspBuilder(RELAY_URL)
    .withName(userName.value || 'demo-user')
    .withReconnect(true)

  if (authToken.value) {
    builder.withToken(authToken.value)
  }

  const c = await builder.connect()

  client.value = c
  connected.value = true
  sessionId.value = c.session || null
  error.value = null

  c.onDisconnect(() => {
    connected.value = false
    scheduleReconnect()
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

function disconnect() {
  clearTimeout(reconnectTimer)
  clearInterval(heartbeatTimer)
  if (client.value) {
    client.value.close()
    client.value = null
  }
  connected.value = false
}

function scheduleReconnect() {
  clearTimeout(reconnectTimer)
  reconnectTimer = setTimeout(() => {
    if (!connected.value && authToken.value) {
      connect().catch(() => scheduleReconnect())
    }
  }, 3000)
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
