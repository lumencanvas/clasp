import { ref, shallowRef, readonly } from 'vue'
import { ClaspBuilder } from '@clasp-to/core'
import { DEFAULT_RELAY_URL } from '../lib/constants.js'

// Shared state across all components
const client = shallowRef(null)
const connected = ref(false)
const connecting = ref(false)
const error = ref(null)
const sessionId = ref(null)

// Persisted settings
const savedUrl = localStorage.getItem('clasp-chat-url')

const url = ref(savedUrl || DEFAULT_RELAY_URL)

async function connect(displayName) {
  if (connecting.value || connected.value) return
  connecting.value = true
  error.value = null

  try {
    localStorage.setItem('clasp-chat-url', url.value)

    const builder = new ClaspBuilder(url.value)
      .name(displayName || 'ChatUser')
      .features(['param', 'event', 'stream', 'gesture', 'timeline'])
      .reconnect(true)

    const c = await builder.connect()
    client.value = c
    connected.value = true
    sessionId.value = c.session

    c.onDisconnect(() => {
      connected.value = false
    })

    c.onReconnect(() => {
      connected.value = true
    })

    c.onError((err) => {
      error.value = err.message
    })
  } catch (e) {
    error.value = e.message
  } finally {
    connecting.value = false
  }
}

function disconnect() {
  if (client.value) {
    client.value.close()
    client.value = null
    connected.value = false
    sessionId.value = null
  }
}

function subscribe(pattern, callback) {
  if (!client.value) return () => {}
  return client.value.on(pattern, (value, address, meta) => {
    callback?.(value, address, meta)
  })
}

function subscribeRaw(pattern, callback) {
  if (!client.value) return () => {}
  return client.value.on(pattern, callback)
}

function set(address, value) {
  if (!client.value) return
  client.value.set(address, value)
}

function emit(address, payload) {
  if (!client.value) return
  client.value.emit(address, payload)
}

function stream(address, value) {
  if (!client.value) return
  client.value.stream(address, value)
}

async function get(address) {
  if (!client.value) return undefined
  return await client.value.get(address)
}

function bundle(messages, options) {
  if (!client.value) return
  client.value.bundle(messages, options)
}

function cached(address) {
  return client.value?.cache?.get(address)
}

function time() {
  return client.value?.time() ?? Date.now() * 1000
}

export function useClasp() {
  return {
    client: readonly(client),
    connected: readonly(connected),
    connecting: readonly(connecting),
    error: readonly(error),
    sessionId: readonly(sessionId),
    url,
    connect,
    disconnect,
    subscribe,
    subscribeRaw,
    set,
    emit,
    stream,
    get,
    bundle,
    cached,
    time,
  }
}
