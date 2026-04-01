<script setup>
import { ref, reactive, computed, watch, onMounted, onUnmounted } from 'vue'
import { useRelay } from '../composables/useRelay.js'
import { useToast } from '../composables/useToast.js'
import { useFloodControl } from '../composables/useFloodControl.js'
import { useLiveStream } from '../composables/useLiveStream.js'
import SettingsPanel from '../components/social/SettingsPanel.vue'
import StreamModal from '../components/social/StreamModal.vue'
import LiveStrip from '../components/social/LiveStrip.vue'
import PostCard from '../components/social/PostCard.vue'
import Composer from '../components/social/Composer.vue'
import ToastContainer from '../components/ToastContainer.vue'

const { client, userName, authToken, connect, ensureAuth } = useRelay()
const { toast } = useToast()
const flood = useFloodControl()
const composerRef = ref(null)

// --- Identity (uses global auth, persists extras in rly_me) ---
function loadMe() {
  const saved = localStorage.getItem('rly_me')
  if (saved) try { return JSON.parse(saved) } catch {}
  const id = 'u' + Date.now() + Math.random().toString(36).slice(2, 8)
  return { id, name: '', handle: '' }
}
const me = reactive(loadMe())
function saveMe() { localStorage.setItem('rly_me', JSON.stringify(me)) }

const showSettings = ref(false)

// Sync identity from global auth
watch(userName, (n) => {
  if (n && n !== me.name) {
    me.name = n
    me.handle = '@' + n.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '').slice(0, 20)
    saveMe()
    sendPresence()
  }
}, { immediate: true })

// --- Channel & namespace ---
const channel = ref((location.hash.match(/[#&]ch=([^&]+)/) || [])[1] || 'main')
const NS = computed(() => `/social/v1/${channel.value}`)

// --- Connection state ---
const connState = ref('connecting')
const connLabel = computed(() => ({
  on: 'demo-relay / ' + me.id.slice(0, 16),
  off: 'disconnected -- reconnecting',
  rc: 'reconnecting...',
  er: 'relay unreachable',
}[connState.value] || 'connecting...'))

// --- Posts ---
const posts = reactive(new Map())
const sortedPosts = computed(() => [...posts.values()].sort((a, b) => b.created - a.created))
const ageTick = ref(0)
let postSeq = 0

function addPost(p) {
  if (posts.has(p.id)) return
  p.reactions = p.reactions || { zap: 0, rep: 0, heart: 0 }
  p.myReactions = p.myReactions || {}
  posts.set(p.id, p)
  if (posts.size > 80) {
    const sorted = [...posts.entries()].sort((a, b) => a[1].created - b[1].created)
    for (let i = 0; i < sorted.length - 70; i++) posts.delete(sorted[i][0])
  }
}
function removePost(id) { posts.delete(id) }

// --- Presence ---
const presence = reactive(new Map())
const onlineCount = computed(() => presence.size + 1)
let presenceTimer = null

function sendPresence() {
  const c = client.value
  if (!c || !me.name) return
  try { c.set(`${NS.value}/pres/${me.id}`, JSON.stringify({ id: me.id, name: me.name, handle: me.handle, ts: Date.now() }), { ttl: 35 }) } catch {}
}
function clearPresence() {
  const c = client.value
  if (!c) return
  try { c.set(`${NS.value}/pres/${me.id}`, null) } catch {}
}

// --- Live streaming ---
const live = useLiveStream(
  () => client.value,
  () => NS.value,
  () => me,
)

// --- Post submission ---
function sendToRelay(p) {
  addPost(p)
  const c = client.value
  if (c) {
    const payload = { ...p }; delete payload.myReactions
    const ttlOpts = p.ttl === 0 ? { ttl: 0 } : { ttl: p.ttl, absolute: true }
    c.set(`${NS.value}/post/${p.id}`, JSON.stringify(payload), ttlOpts)
  }
}

function handleTransmit(opts) {
  const p = {
    id: me.id + '_' + (++postSeq) + '_' + Date.now(),
    author: { id: me.id, name: me.name, handle: me.handle },
    text: opts.text || '', image: opts.image || null,
    stype: opts.stype || 'SET',
    created: Date.now(), ttl: opts.ttl ?? 1800,
    reactions: { zap: 0, rep: 0, heart: 0 }, myReactions: {},
  }
  flood.submit(p, sendToRelay)
}

function deletePost(id) {
  removePost(id)
  const c = client.value
  if (c) c.set(`${NS.value}/post/${id}`, null)
}

function toggleReaction(postId, key) {
  const p = posts.get(postId)
  if (!p) return
  p.reactions = p.reactions || { zap: 0, rep: 0, heart: 0 }
  p.myReactions = p.myReactions || {}
  if (p.myReactions[key]) {
    p.reactions[key] = Math.max(0, (p.reactions[key] || 0) - 1)
    delete p.myReactions[key]
  } else {
    p.reactions[key] = (p.reactions[key] || 0) + 1
    p.myReactions[key] = true
    const c = client.value
    if (c) c.emit(`${NS.value}/react/${postId}`, JSON.stringify({ postId, reaction: key, userId: me.id }))
  }
}

// --- Go live handler ---
async function handleGoLive() {
  const started = await live.goLive(toast)
  if (started && composerRef.value) {
    composerRef.value.transmit({ text: 'started a live stream', stype: 'STREAM' })
    toast('you are live on #' + channel.value)
  }
}

// --- Settings ---
function handleSaveSettings({ name, handle }) {
  if (!name) { toast('name cannot be empty', 'err'); return }
  me.name = name
  me.handle = '@' + (handle || name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '').slice(0, 20))
  saveMe(); sendPresence()
  showSettings.value = false
  toast('identity saved')
}

// --- CLASP subscriptions ---
const unsubs = []

function setupSubscriptions() {
  const c = client.value
  if (!c) { console.warn('[social] setupSubscriptions: no client'); return }
  console.log('[social] subscribing to', `${NS.value}/post/**`)

  const u1 = c.on(`${NS.value}/post/**`, (v, addr) => {
    if (!v) { removePost(addr.split('/').pop()); return }
    try {
      const p = JSON.parse(v)
      if (p.author) {
        addPost(p)
        if (posts.size <= 3) console.log('[social] post received:', p.id?.slice(-20), 'total:', posts.size)
      }
    } catch (e) { console.warn('[social] parse error:', e) }
  })
  if (typeof u1 === 'function') unsubs.push(u1)
  else console.warn('[social] subscription returned non-function:', typeof u1)

  const u2 = c.on(`${NS.value}/pres/**`, (v, addr) => {
    const uid = addr.slice(`${NS.value}/pres/`.length)
    if (uid === me.id) return
    if (!v) presence.delete(uid)
    else { try { presence.set(uid, JSON.parse(v)) } catch {} }
  })
  if (typeof u2 === 'function') unsubs.push(u2)

  const u3 = c.on(`${NS.value}/react/**`, (v) => {
    if (!v) return
    try {
      const d = JSON.parse(v)
      if (d && d.userId !== me.id && posts.has(d.postId)) {
        const p = posts.get(d.postId)
        p.reactions = p.reactions || {}
        p.reactions[d.reaction] = (p.reactions[d.reaction] || 0) + 1
      }
    } catch {}
  })
  if (typeof u3 === 'function') unsubs.push(u3)

  // Live stream subscriptions
  const liveUnsubs = live.subscribe(toast)
  unsubs.push(...liveUnsubs)
}

// --- Lifecycle ---
let ageTimer = null, expiryTimer = null

onMounted(async () => {
  try {
    console.log('[social] ensureAuth...')
    await ensureAuth(me.name || 'anon')
    console.log('[social] auth ok, token:', authToken.value?.slice(0, 15))
    // Sync name from auth into local identity
    if (userName.value && !me.name) {
      me.name = userName.value
      me.handle = '@' + userName.value.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '').slice(0, 20)
    }
    // If still no name, derive from id
    if (!me.name) {
      me.name = 'anon_' + me.id.slice(2, 8)
      me.handle = '@' + me.id.slice(2, 10)
    }
    saveMe()

    console.log('[social] connecting...')
    await connect()
    const c0 = client.value
    console.log('[social] connected, session:', c0?.session)

    // RAW WebSocket message spy -- see every message from relay
    if (c0?.inner?.ws || c0?.ws) {
      const ws = c0.inner?.ws || c0.ws
      const origHandler = ws.onmessage
      ws.onmessage = (ev) => {
        const bytes = new Uint8Array(ev.data)
        // Message type is after frame header. Frame: [magic][flags][len_hi][len_lo][...payload]
        // Payload starts at offset 4+ depending on frame format. Just log raw size + first bytes.
        const msgTypes = { 0x01: 'HELLO', 0x02: 'WELCOME', 0x03: 'ANNOUNCE', 0x10: 'SUBSCRIBE', 0x21: 'SET', 0x23: 'SNAPSHOT', 0x20: 'PUBLISH', 0x30: 'BUNDLE', 0x50: 'ACK', 0x51: 'ERROR' }
        // Try to find message type byte (skip frame header)
        let typeByte = bytes.length > 4 ? bytes[4] : bytes[0]
        if (bytes[0] === 0xC1) typeByte = bytes[4] // frame magic
        console.log('[ws-raw]', msgTypes[typeByte] || ('0x' + typeByte.toString(16)), bytes.length + 'B')
        if (origHandler) origHandler.call(ws, ev)
      }
      console.log('[social] ws spy installed')
    } else {
      console.warn('[social] could not install ws spy - client structure:', Object.keys(c0 || {}))
    }

    connState.value = 'on'
    setupSubscriptions()
    sendPresence()
    setTimeout(() => console.log('[social] posts after 3s:', posts.size), 3000)
    presenceTimer = setInterval(sendPresence, 28000)

    const c = client.value
    if (c) {
      c.onConnect(() => { connState.value = 'on'; sendPresence(); live.republishLive() })
      c.onDisconnect(() => { connState.value = 'off' })
      c.onReconnect(() => { connState.value = 'rc' })
    }
  } catch {
    connState.value = 'er'
    toast('relay unreachable', 'err')
  }

  ageTimer = setInterval(() => { ageTick.value++ }, 5000)
  expiryTimer = setInterval(() => {
    const now = Date.now()
    posts.forEach((p, id) => { if (p.ttl && (now - p.created) / 1000 > p.ttl) removePost(id) })
  }, 10000)

  document.addEventListener('visibilitychange', onVisibility)
  window.addEventListener('beforeunload', clearPresence)
})

function onVisibility() {
  if (document.hidden) {
    clearPresence()
    live.clearLive()
  } else {
    sendPresence()
    live.republishLive()
  }
}

onUnmounted(() => {
  clearInterval(ageTimer); clearInterval(expiryTimer); clearInterval(presenceTimer)
  flood.stop()
  unsubs.forEach(u => { try { u() } catch {} })
  live.cleanup()
  clearPresence()
  document.removeEventListener('visibilitychange', onVisibility)
  window.removeEventListener('beforeunload', clearPresence)
})
</script>

<template>
  <div class="social">
    <!-- TOPBAR -->
    <header class="topbar">
      <div class="tl">
        <span class="tl-word">RELAY</span>
        <span class="ch-tag">#{{ channel }}</span>
      </div>
      <div class="tc">
        <div class="cdot" :class="connState"></div>
        <span class="conn-lbl">{{ connLabel }}</span>
      </div>
      <div class="tr">
        <span class="online-ct">{{ onlineCount }} online</span>
        <button class="ib" @click="showSettings = true" title="settings">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="15" height="15"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06a1.65 1.65 0 00-.33 1.82 1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/></svg>
        </button>
      </div>
    </header>

    <main class="app">
      <LiveStrip
        :streams="live.streams.value"
        :is-live="live.isLive.value"
        :my-name="me.name"
        @open-self="live.openSelf()"
        @open-viewer="live.openViewer($event)"
      />

      <!-- Flood banner -->
      <div v-if="flood.active.value" class="flood">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="13" height="13"><path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
        <span>dampening -- {{ flood.queue.value.length }} queued</span>
        <div class="ftk"><div class="ffl" :style="{ width: Math.min(100, flood.queue.value.length / flood.RATE_MAX * 100) + '%' }"></div></div>
      </div>

      <Composer
        ref="composerRef"
        :handle="me.handle"
        :is-live="live.isLive.value"
        @transmit="handleTransmit"
        @go-live="handleGoLive"
        @error="(msg) => toast(msg, 'err')"
      />

      <!-- Feed header -->
      <div class="fhd">
        <div class="flbl">signal feed</div>
        <div class="fln"></div>
        <div class="fct">{{ posts.size }} signals</div>
      </div>

      <!-- Queued posts -->
      <div v-for="q in flood.queue.value" :key="q.id" class="qi">
        <div class="qdot"></div>
        <span>{{ (q.text || 'post').slice(0, 55) }}</span>
        <span class="qt">queued</span>
      </div>

      <!-- Feed -->
      <PostCard
        v-for="p in sortedPosts"
        :key="p.id"
        :post="p"
        :is-own="p.author?.id === me.id"
        :age-tick="ageTick"
        @react="toggleReaction"
        @delete="deletePost"
      />

      <div v-if="!sortedPosts.length" class="fempty">
        <p>no signals on #{{ channel }}<br>transmit to start the feed</p>
      </div>
    </main>

    <StreamModal
      :show="live.showModal.value"
      :meta="live.modalMeta"
      :status="live.streamStatus.value"
      :viewer-count="live.viewerCount.value"
      @video-ready="(el) => live.setVideoEl(el)"
      @close="live.closeModal()"
      @end="live.stopLive(toast)"
    />

    <SettingsPanel
      :show="showSettings"
      :name="me.name"
      :handle="me.handle"
      @close="showSettings = false"
      @save="handleSaveSettings"
    />

    <ToastContainer />
  </div>
</template>

<style src="./social-styles.css"></style>
