<script setup>
import { ref, reactive, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRelay } from '../composables/useRelay.js'

const emit = defineEmits(['auth-required'])
const { client, connected, userName, authToken, connect } = useRelay()

// --- Identity ---
function loadMe() {
  const saved = localStorage.getItem('rly_me')
  if (saved) try { return JSON.parse(saved) } catch {}
  const id = 'u' + Date.now() + Math.random().toString(36).slice(2, 8)
  return { id, name: 'anon_' + id.slice(2, 8), handle: '@' + id.slice(2, 10) }
}
const me = reactive(loadMe())
function saveMe() { localStorage.setItem('rly_me', JSON.stringify(me)) }
const showNamePicker = ref(me.name.startsWith('anon_'))
const showSettings = ref(false)
const settingsName = ref('')
const settingsHandle = ref('')

// --- Channel ---
const channel = ref((location.hash.match(/[#&]ch=([^&]+)/) || [])[1] || 'main')
const NS = computed(() => `/social/v1/${channel.value}`)

// --- State ---
const posts = reactive(new Map())
const streams = reactive(new Map())
const presence = reactive(new Map())
const onlineCount = computed(() => presence.size + 1)
const sortedPosts = computed(() => [...posts.values()].sort((a, b) => b.created - a.created))
const toasts = ref([])

// --- Connection state ---
const connState = ref('connecting') // on, off, rc, er
const connLabel = computed(() => {
  const m = { on: 'demo-relay / ' + me.id.slice(0, 16), off: 'disconnected -- reconnecting', rc: 'reconnecting...', er: 'relay unreachable' }
  return m[connState.value] || 'connecting...'
})

// --- Live streaming ---
const isLive = ref(false)
let myStream = null
const viewerPCs = new Map()
const pendVIce = new Map()
let viewerPC = null
let watchId = null
const pendSIce = []
const ICE = [{ urls: 'stun:stun.l.google.com:19302' }, { urls: 'stun:stun1.l.google.com:19302' }]
const showStreamModal = ref(false)
const streamModalMeta = reactive({ name: '', sub: '', isSelf: false })
const streamStatus = ref('connecting')
const viewerCount = ref(0)
const streamVideoRef = ref(null)
const liveCount = computed(() => streams.size + (isLive.value ? 1 : 0))

// --- Composer ---
const composerText = ref('')
const selectedTtl = ref(1800)
const compressedImage = ref(null)
const imageInput = ref(null)
const showUrlRow = ref(false)
const urlInput = ref('')
const maxChars = 500
const charCount = computed(() => maxChars - composerText.value.length)
const charClass = computed(() => {
  const n = composerText.value.length
  if (n > maxChars) return 'ovr'
  if (n > maxChars * 0.85) return 'wrn'
  return ''
})
const ttlOptions = [
  { label: '5m', value: 300 },
  { label: '30m', value: 1800 },
  { label: '1h', value: 3600 },
  { label: '24h', value: 86400 },
  { label: 'never', value: 0 },
]

// --- Rate limiting ---
const postTimestamps = []
const RATE_MAX = 8, RATE_WINDOW = 10000, DRAIN_MS = 1900
const floodQueue = ref([])
const showFlood = ref(false)
let drainTimer = null
let postSeq = 0

function canPost() {
  const now = Date.now()
  while (postTimestamps.length && now - postTimestamps[0] > RATE_WINDOW) postTimestamps.shift()
  return postTimestamps.length < RATE_MAX
}

// --- Toast system ---
function toast(msg, type) {
  const id = Date.now() + Math.random()
  toasts.value.push({ id, msg, type, out: false })
  setTimeout(() => {
    const t = toasts.value.find(x => x.id === id)
    if (t) t.out = true
    setTimeout(() => { toasts.value = toasts.value.filter(x => x.id !== id) }, 280)
  }, 3200)
}

// --- Image compression ---
function compressImage(file) {
  return new Promise((resolve, reject) => {
    const rd = new FileReader()
    rd.onerror = reject
    rd.onload = (e) => {
      const img = new Image()
      img.onerror = reject
      img.onload = () => {
        let w = img.width, h = img.height
        const ratio = Math.min(720 / w, 720 / h, 1)
        w = Math.round(w * ratio); h = Math.round(h * ratio)
        const cv = document.createElement('canvas')
        cv.width = w; cv.height = h
        cv.getContext('2d').drawImage(img, 0, 0, w, h)
        function tryQ(q) {
          const d = cv.toDataURL('image/jpeg', q)
          if (d.length * 0.75 > 62000 && q > 0.2) return tryQ(q - 0.15)
          return d
        }
        resolve(tryQ(0.6))
      }
      img.src = e.target.result
    }
    rd.readAsDataURL(file)
  })
}

async function handleImageUpload(e) {
  const f = e.target.files?.[0]
  if (!f) return
  try { compressedImage.value = await compressImage(f) }
  catch { toast('could not load image', 'err') }
  if (imageInput.value) imageInput.value.value = ''
}

function handleUrlInput(e) {
  if (e.key === 'Enter') {
    const u = urlInput.value.trim()
    if (u) { compressedImage.value = u; showUrlRow.value = false }
  }
}

function handleUrlPaste() {
  const u = urlInput.value.trim()
  if (u.match(/^https?:\/\//)) compressedImage.value = u
}

function clearImage() {
  compressedImage.value = null
  urlInput.value = ''
  if (imageInput.value) imageInput.value.value = ''
}

// --- Post management ---
function addPost(p) {
  if (posts.has(p.id)) return
  p.reactions = p.reactions || { zap: 0, rep: 0, heart: 0 }
  p.myReactions = p.myReactions || {}
  posts.set(p.id, p)
  // Trim old posts
  if (posts.size > 80) {
    const sorted = [...posts.entries()].sort((a, b) => a[1].created - b[1].created)
    for (let i = 0; i < sorted.length - 70; i++) posts.delete(sorted[i][0])
  }
}

function removePost(id) { posts.delete(id) }

function submitPost(p) {
  if (canPost()) {
    postTimestamps.push(Date.now())
    addPost(p)
    const c = client.value
    if (c) {
      const payload = { ...p }; delete payload.myReactions
      c.set(`${NS.value}/post/${p.id}`, JSON.stringify(payload), { ttl: p.ttl || undefined, absolute: true })
    }
    if (!floodQueue.value.length) showFlood.value = false
  } else {
    floodQueue.value.push(p)
    showFlood.value = true
    startDrain()
  }
}

function startDrain() {
  if (drainTimer) return
  drainTimer = setInterval(() => {
    if (!floodQueue.value.length) { clearInterval(drainTimer); drainTimer = null; showFlood.value = false; return }
    const p = floodQueue.value.shift()
    postTimestamps.push(Date.now())
    addPost(p)
    const c = client.value
    if (c) {
      const payload = { ...p }; delete payload.myReactions
      c.set(`${NS.value}/post/${p.id}`, JSON.stringify(payload), { ttl: p.ttl || undefined, absolute: true })
    }
  }, DRAIN_MS)
}

function doTransmit(opts = {}) {
  const text = (opts.text || composerText.value.trim()).slice(0, maxChars)
  if (!text && !compressedImage.value && !opts.stype) return
  const p = {
    id: me.id + '_' + (++postSeq) + '_' + Date.now(),
    author: { id: me.id, name: me.name, handle: me.handle },
    text, image: opts.image || compressedImage.value || null,
    stype: opts.stype || 'SET',
    created: Date.now(), ttl: selectedTtl.value,
    reactions: { zap: 0, rep: 0, heart: 0 }, myReactions: {},
  }
  submitPost(p)
  if (!opts.text) { composerText.value = ''; clearImage(); showUrlRow.value = false }
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

// --- Time formatting ---
function fmtAge(ts) {
  const s = Math.floor((Date.now() - ts) / 1000)
  if (s < 60) return s + 's'
  if (s < 3600) return Math.floor(s / 60) + 'm'
  if (s < 86400) return Math.floor(s / 3600) + 'h'
  return Math.floor(s / 86400) + 'd'
}
function expiryPct(p) {
  if (!p.ttl) return 100
  return Math.max(0, Math.min(100, (1 - (Date.now() - p.created) / (p.ttl * 1000)) * 100))
}

const ageTick = ref(0)

// --- Presence ---
let presenceTimer = null
function sendPresence() {
  const c = client.value
  if (!c) return
  try { c.set(`${NS.value}/pres/${me.id}`, JSON.stringify({ id: me.id, name: me.name, handle: me.handle, ts: Date.now() }), { ttl: 35 }) } catch {}
}
function clearPresence() {
  const c = client.value
  if (!c) return
  try {
    c.set(`${NS.value}/pres/${me.id}`, null)
    if (isLive.value) c.set(`${NS.value}/live/${me.id}`, null)
  } catch {}
}

// --- WebRTC: Broadcaster ---
async function handleWatchReq(viewerId) {
  if (!isLive.value || !myStream) return
  if (viewerPCs.has(viewerId)) { viewerPCs.get(viewerId).close(); viewerPCs.delete(viewerId) }
  const pc = new RTCPeerConnection({ iceServers: ICE })
  viewerPCs.set(viewerId, pc)
  myStream.getTracks().forEach(t => pc.addTrack(t, myStream))
  pc.onicecandidate = (e) => {
    if (!e.candidate) return
    const c = client.value
    if (c) c.emit(`${NS.value}/ice/${viewerId}`, JSON.stringify({ fromId: me.id, candidate: e.candidate.toJSON() }))
  }
  pc.onconnectionstatechange = () => {
    if (['failed', 'closed', 'disconnected'].includes(pc.connectionState)) viewerPCs.delete(viewerId)
    updateViewerCount()
  }
  try {
    const offer = await pc.createOffer()
    await pc.setLocalDescription(offer)
    const c = client.value
    if (c) c.emit(`${NS.value}/offer/${viewerId}`, JSON.stringify({ streamerId: me.id, sdp: pc.localDescription.toJSON() }))
    const buf = pendVIce.get(viewerId) || []; pendVIce.delete(viewerId)
    for (const ice of buf) pc.addIceCandidate(new RTCIceCandidate(ice)).catch(() => {})
  } catch (e) { console.error('[offer]', e) }
}

async function handleAnswer(vid, sdp) {
  const pc = viewerPCs.get(vid)
  if (!pc) return
  try {
    await pc.setRemoteDescription(new RTCSessionDescription(sdp))
    const buf = pendVIce.get(vid) || []; pendVIce.delete(vid)
    for (const ice of buf) pc.addIceCandidate(new RTCIceCandidate(ice)).catch(() => {})
  } catch (e) { console.error('[answer]', e) }
  updateViewerCount()
}

function updateViewerCount() {
  let n = 0
  viewerPCs.forEach(p => { if (p.connectionState === 'connected') n++ })
  viewerCount.value = n
}

// --- WebRTC: Viewer ---
async function handleOffer(streamerId, sdp) {
  if (viewerPC) { viewerPC.close(); viewerPC = null }
  watchId = streamerId
  const pc = new RTCPeerConnection({ iceServers: ICE })
  viewerPC = pc
  pc.ontrack = (ev) => {
    if (streamVideoRef.value) { streamVideoRef.value.srcObject = ev.streams[0] }
    streamStatus.value = 'live'
  }
  pc.onicecandidate = (e) => {
    if (!e.candidate) return
    const c = client.value
    if (c) c.emit(`${NS.value}/ice/${streamerId}`, JSON.stringify({ fromId: me.id, candidate: e.candidate.toJSON() }))
  }
  pc.onconnectionstatechange = () => {
    if (pc.connectionState === 'connected') streamStatus.value = 'live'
    else if (pc.connectionState === 'failed') streamStatus.value = 'lost'
  }
  try {
    await pc.setRemoteDescription(new RTCSessionDescription(sdp))
    const ans = await pc.createAnswer()
    await pc.setLocalDescription(ans)
    const c = client.value
    if (c) c.emit(`${NS.value}/answer/${streamerId}`, JSON.stringify({ viewerId: me.id, sdp: pc.localDescription.toJSON() }))
    for (const ice of pendSIce) pc.addIceCandidate(new RTCIceCandidate(ice)).catch(() => {})
    pendSIce.length = 0
  } catch (e) { console.error('[viewer]', e) }
}

function handleIce(fromId, candidate) {
  if (viewerPCs.has(fromId)) {
    const pc = viewerPCs.get(fromId)
    if (pc.remoteDescription) pc.addIceCandidate(new RTCIceCandidate(candidate)).catch(() => {})
    else { const buf = pendVIce.get(fromId) || []; buf.push(candidate); pendVIce.set(fromId, buf) }
  } else if (viewerPC && fromId === watchId) {
    if (viewerPC.remoteDescription) viewerPC.addIceCandidate(new RTCIceCandidate(candidate)).catch(() => {})
    else pendSIce.push(candidate)
  }
}

// --- Live: Go live / Stop ---
async function goLive() {
  if (isLive.value) { openSelf(); return }
  try {
    const stream = await navigator.mediaDevices.getUserMedia({ video: { facingMode: 'user', width: { ideal: 1280 }, height: { ideal: 720 } }, audio: true })
    myStream = stream
    isLive.value = true
    const c = client.value
    if (c) c.set(`${NS.value}/live/${me.id}`, JSON.stringify({ userId: me.id, name: me.name, handle: me.handle }), { ttl: 35 })
    doTransmit({ text: 'started a live stream', stype: 'STREAM' })
    openSelf()
    toast('you are live on #' + channel.value)
  } catch (e) {
    const msgs = { NotAllowedError: 'camera permission denied', NotFoundError: 'no camera found', NotReadableError: 'camera in use', OverconstrainedError: 'resolution unsupported' }
    toast(msgs[e.name] || 'camera error', 'err')
  }
}

function stopLive() {
  if (!isLive.value) return
  if (myStream) { myStream.getTracks().forEach(t => t.stop()); myStream = null }
  viewerPCs.forEach(p => p.close()); viewerPCs.clear(); pendVIce.clear()
  isLive.value = false
  const c = client.value
  if (c) c.set(`${NS.value}/live/${me.id}`, null)
  showStreamModal.value = false
  toast('stream ended')
}

function openSelf() {
  if (!isLive.value || !myStream) return
  streamModalMeta.name = me.name; streamModalMeta.sub = 'your broadcast'; streamModalMeta.isSelf = true
  streamStatus.value = 'broadcasting'
  showStreamModal.value = true
  setTimeout(() => { if (streamVideoRef.value) { streamVideoRef.value.srcObject = myStream; streamVideoRef.value.muted = true } }, 50)
}

function openViewer(entry) {
  if (viewerPC) { viewerPC.close(); viewerPC = null }
  watchId = entry.userId
  streamModalMeta.name = entry.name; streamModalMeta.sub = 'live stream'; streamModalMeta.isSelf = false
  streamStatus.value = 'connecting'
  showStreamModal.value = true
  setTimeout(() => { if (streamVideoRef.value) { streamVideoRef.value.srcObject = null; streamVideoRef.value.muted = false } }, 50)
  const c = client.value
  if (c) c.emit(`${NS.value}/watch/${entry.userId}`, JSON.stringify({ viewerId: me.id }))
}

function closeStreamModal() {
  showStreamModal.value = false
  if (!isLive.value && viewerPC) { viewerPC.close(); viewerPC = null; watchId = null }
  if (streamVideoRef.value) streamVideoRef.value.srcObject = null
}

// --- Settings ---
function openSettings() {
  settingsName.value = me.name
  settingsHandle.value = me.handle.replace(/^@/, '')
  showSettings.value = true
}
function saveSettings() {
  const n = settingsName.value.trim()
  if (!n) { toast('name cannot be empty', 'err'); return }
  me.name = n
  const h = settingsHandle.value.trim().replace(/^@/, '')
  me.handle = '@' + (h || n.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '').slice(0, 20))
  saveMe(); sendPresence()
  showSettings.value = false
  toast('identity saved')
}

// --- Name picker ---
const pickerName = ref('')
function commitName() {
  const n = pickerName.value.trim()
  if (n) {
    me.name = n
    me.handle = '@' + n.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '').slice(0, 20)
    saveMe()
  }
  showNamePicker.value = false
  setTimeout(sendPresence, 400)
}

// --- CLASP subscriptions ---
const unsubs = []
function setupSubscriptions() {
  const c = client.value
  if (!c) return

  const u1 = c.on(`${NS.value}/post/**`, (v, addr) => {
    if (!v) { removePost(addr.split('/').pop()); return }
    try { const p = JSON.parse(v); if (p.author) addPost(p) } catch {}
  })
  if (typeof u1 === 'function') unsubs.push(u1)

  const u2 = c.on(`${NS.value}/pres/**`, (v, addr) => {
    const uid = addr.slice(`${NS.value}/pres/`.length)
    if (uid === me.id) return
    if (!v) { presence.delete(uid) } else { try { presence.set(uid, JSON.parse(v)) } catch {} }
  })
  if (typeof u2 === 'function') unsubs.push(u2)

  const u3 = c.on(`${NS.value}/live/**`, (v, addr) => {
    const uid = addr.slice(`${NS.value}/live/`.length)
    if (uid === me.id) return
    if (!v) {
      streams.delete(uid)
      if (viewerPC && watchId === uid) { viewerPC.close(); viewerPC = null; watchId = null; closeStreamModal() }
    } else {
      try {
        const e = JSON.parse(v)
        if (!streams.has(e.userId)) { streams.set(e.userId, e); toast(e.name + ' went live') }
      } catch {}
    }
  })
  if (typeof u3 === 'function') unsubs.push(u3)

  const u4 = c.on(`${NS.value}/react/**`, (v) => {
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
  if (typeof u4 === 'function') unsubs.push(u4)

  // WebRTC signaling
  const u5 = c.on(`${NS.value}/watch/${me.id}`, (v) => {
    if (!v || !isLive.value) return
    try { handleWatchReq(JSON.parse(v).viewerId) } catch {}
  })
  if (typeof u5 === 'function') unsubs.push(u5)

  const u6 = c.on(`${NS.value}/offer/${me.id}`, (v) => {
    if (!v) return
    try { const d = JSON.parse(v); handleOffer(d.streamerId, d.sdp) } catch (e) { console.error('[offer]', e) }
  })
  if (typeof u6 === 'function') unsubs.push(u6)

  const u7 = c.on(`${NS.value}/answer/${me.id}`, (v) => {
    if (!v) return
    try { const d = JSON.parse(v); handleAnswer(d.viewerId, d.sdp) } catch {}
  })
  if (typeof u7 === 'function') unsubs.push(u7)

  const u8 = c.on(`${NS.value}/ice/${me.id}`, (v) => {
    if (!v) return
    try { const d = JSON.parse(v); handleIce(d.fromId, d.candidate) } catch {}
  })
  if (typeof u8 === 'function') unsubs.push(u8)
}

// --- Lifecycle ---
let ageTimer = null
let expiryTimer = null

onMounted(async () => {
  if (!authToken.value) { emit('auth-required'); return }
  me.name = userName.value || me.name
  saveMe()

  try {
    await connect()
    connState.value = 'on'
    setupSubscriptions()
    sendPresence()
    presenceTimer = setInterval(sendPresence, 28000)

    const c = client.value
    if (c) {
      c.onConnect(() => {
        connState.value = 'on'; sendPresence()
        if (isLive.value) c.set(`${NS.value}/live/${me.id}`, JSON.stringify({ userId: me.id, name: me.name, handle: me.handle }), { ttl: 35 })
      })
      c.onDisconnect(() => { connState.value = 'off' })
      c.onReconnect(() => { connState.value = 'rc' })
    }
  } catch (e) {
    connState.value = 'er'
    toast('relay unreachable', 'err')
  }

  ageTimer = setInterval(() => { ageTick.value++ }, 5000)
  expiryTimer = setInterval(() => {
    const now = Date.now()
    posts.forEach((p, id) => { if (p.ttl && (now - p.created) / 1000 > p.ttl) removePost(id) })
  }, 10000)

  const onVis = () => { if (document.hidden) clearPresence(); else { sendPresence(); if (isLive.value) { const c = client.value; if (c) c.set(`${NS.value}/live/${me.id}`, JSON.stringify({ userId: me.id, name: me.name, handle: me.handle }), { ttl: 35 }) } } }
  document.addEventListener('visibilitychange', onVis)
  window.addEventListener('beforeunload', clearPresence)
})

onUnmounted(() => {
  clearInterval(ageTimer)
  clearInterval(expiryTimer)
  clearInterval(presenceTimer)
  clearInterval(drainTimer)
  unsubs.forEach(u => { try { u() } catch {} })
  if (isLive.value) stopLive()
  clearPresence()
})
</script>

<template>
  <div class="social">
    <!-- NAME PICKER -->
    <div v-if="showNamePicker" class="npick">
      <div class="npbox">
        <div class="npttl">RELAY</div>
        <div class="npsub">ephemeral signals on the CLASP protocol<br>pick a name to get started</div>
        <div class="nprow">
          <label>your name</label>
          <input v-model="pickerName" placeholder="enter a display name" maxlength="32" @keydown.enter="commitName" autofocus />
        </div>
        <button class="npbtn" @click="commitName">JOIN THE RELAY</button>
        <button class="npskip" @click="showNamePicker = false">skip -- stay anonymous</button>
      </div>
    </div>

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
        <button class="ib" @click="openSettings" title="settings">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="15" height="15"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06a1.65 1.65 0 00-.33 1.82 1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/></svg>
        </button>
      </div>
    </header>

    <main class="app">
      <!-- LIVE STRIP -->
      <div class="ls">
        <div class="ls-hd">
          <div class="lbdge"><div class="lbdot"></div>LIVE</div>
          <span class="live-ct">{{ liveCount }} live</span>
        </div>
        <div class="ls-sc">
          <button v-if="isLive" class="lp own" @click="openSelf">
            <div class="lp-dot"></div>{{ me.name }}
          </button>
          <button v-for="[uid, entry] in streams" :key="uid" class="lp" @click="openViewer(entry)">
            <div class="lp-dot"></div>{{ entry.name }}
          </button>
          <div v-if="!liveCount" class="ls-em">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" width="13" height="13"><line x1="1" y1="1" x2="23" y2="23"/><path d="M16.72 11.06A10.94 10.94 0 0119 12.55"/><path d="M5 12.55a11 11 0 0110.11-2.97"/></svg>
            no active streams
          </div>
        </div>
      </div>

      <!-- FLOOD BANNER -->
      <div v-if="showFlood" class="flood">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="13" height="13"><path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
        <span>dampening -- {{ floodQueue.length }} queued</span>
        <div class="ftk"><div class="ffl" :style="{ width: Math.min(100, floodQueue.length / RATE_MAX * 100) + '%' }"></div></div>
      </div>

      <!-- COMPOSER -->
      <div class="composer">
        <div class="ct">
          <div class="cav">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" width="15" height="15"><circle cx="12" cy="8" r="4"/><path d="M4 20c0-4 3.6-7 8-7s8 3 8 7"/></svg>
          </div>
          <div class="cr">
            <div class="chandl">{{ me.handle }}</div>
            <textarea v-model="composerText" placeholder="transmit a signal..." rows="2" @keydown.meta.enter.prevent="doTransmit()" @keydown.ctrl.enter.prevent="doTransmit()"></textarea>
          </div>
        </div>

        <div v-if="compressedImage" class="img-prev">
          <img :src="compressedImage" @error="clearImage" />
          <button class="img-prev-rm" @click="clearImage">&times;</button>
        </div>

        <div v-if="showUrlRow" class="url-row">
          <input v-model="urlInput" placeholder="paste image URL and press enter" @keydown="handleUrlInput" @input="handleUrlPaste" />
        </div>

        <div class="ttl-row">
          <span class="ttl-lbl">ttl</span>
          <button v-for="o in ttlOptions" :key="o.value" class="ttb" :class="{ on: selectedTtl === o.value }" @click="selectedTtl = o.value">{{ o.label }}</button>
        </div>

        <div class="ca">
          <label class="ctool">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="13" height="13"><path d="M23 19a2 2 0 01-2 2H3a2 2 0 01-2-2V8a2 2 0 012-2h4l2-3h6l2 3h4a2 2 0 012 2z"/><circle cx="12" cy="13" r="4"/></svg>
            photo
            <input ref="imageInput" type="file" accept="image/*" style="display:none" @change="handleImageUpload" />
          </label>
          <button class="ctool" @click="showUrlRow = !showUrlRow">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="13" height="13"><path d="M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71"/></svg>
            url
          </button>
          <button class="lbtn" :class="{ on: isLive }" @click="goLive">
            <svg width="10" height="10" viewBox="0 0 24 24" fill="currentColor"><circle cx="12" cy="12" r="9"/></svg>
            <span>{{ isLive ? 'you are live' : 'go live' }}</span>
          </button>
          <span class="cc" :class="charClass">{{ charCount }}</span>
          <button class="txbtn" @click="doTransmit()">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="13" height="13"><line x1="22" y1="2" x2="11" y2="13"/><polygon points="22 2 15 22 11 13 2 9 22 2"/></svg>
            TRANSMIT
          </button>
        </div>
      </div>

      <!-- FEED HEADER -->
      <div class="fhd">
        <div class="flbl">signal feed</div>
        <div class="fln"></div>
        <div class="fct">{{ posts.size }} signals</div>
      </div>

      <!-- QUEUED -->
      <div v-for="q in floodQueue" :key="q.id" class="qi">
        <div class="qdot"></div>
        <span>{{ (q.text || 'post').slice(0, 55) }}</span>
        <span class="qt">queued</span>
      </div>

      <!-- FEED -->
      <article v-for="p in sortedPosts" :key="p.id" class="post" :class="{ exp: p.ttl && expiryPct(p) < 15 }">
        <div class="ph">
          <div class="pav">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" width="14" height="14"><circle cx="12" cy="8" r="4"/><path d="M4 20c0-4 3.6-7 8-7s8 3 8 7"/></svg>
          </div>
          <div class="pm">
            <div class="pn">{{ p.author?.name || 'anon' }}</div>
            <div class="phan">{{ p.author?.handle || '@anon' }}</div>
          </div>
          <div class="pr">
            <span class="stag" :class="(p.stype || 'SET').toLowerCase()">{{ p.stype || 'SET' }}</span>
            <span class="pt" :data-tick="ageTick">{{ fmtAge(p.created) }}</span>
          </div>
        </div>
        <div v-if="p.text" class="pb">{{ p.text }}</div>
        <div v-if="p.image" class="pimg"><img :src="p.image" loading="lazy" @error="$event.target.parentElement.style.display='none'" /></div>
        <div class="pf">
          <button v-for="rk in ['zap', 'rep', 'heart']" :key="rk" class="ab rb" :class="{ act: p.myReactions?.[rk] }" @click="toggleReaction(p.id, rk)">
            <svg v-if="rk === 'zap'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="12" height="12"><polyline points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>
            <svg v-else-if="rk === 'rep'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="12" height="12"><polyline points="17 1 21 5 17 9"/><path d="M3 11V9a4 4 0 014-4h14"/><polyline points="7 23 3 19 7 15"/><path d="M21 13v2a4 4 0 01-4 4H3"/></svg>
            <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="12" height="12"><path d="M20.84 4.61a5.5 5.5 0 00-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 00-7.78 7.78L12 21.23l8.84-8.84a5.5 5.5 0 000-7.78z"/></svg>
            <span class="rc">{{ p.reactions?.[rk] || 0 }}</span>
          </button>
          <button v-if="p.author?.id === me.id" class="ab db" @click="deletePost(p.id)">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="12" height="12"><polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14H6L5 6"/><path d="M10 11v6M14 11v6"/><path d="M9 6V4h6v2"/></svg>
          </button>
        </div>
        <div class="pex"><div class="pexf" :style="{ width: expiryPct(p) + '%', background: (p.ttl && expiryPct(p) < 15) ? 'var(--red)' : undefined }"></div></div>
      </article>

      <div v-if="!sortedPosts.length" class="fempty">
        <p>no signals on #{{ channel }}<br>transmit to start the feed</p>
      </div>
    </main>

    <!-- STREAM MODAL -->
    <div v-if="showStreamModal" class="modal" @click.self="closeStreamModal">
      <div class="mbk" @click="closeStreamModal"></div>
      <div class="mbox">
        <div class="mhd">
          <div class="mttl">
            <div class="mttl-name">{{ streamModalMeta.name }}</div>
            <div class="msub">{{ streamModalMeta.sub }}</div>
          </div>
          <div class="mbdge"><div class="mbdot"></div>LIVE</div>
          <button class="mclose" @click="closeStreamModal">&times;</button>
        </div>
        <div class="mvw">
          <video ref="streamVideoRef" autoplay playsinline></video>
          <div class="shud">
            <div class="slive"><div class="sbdot"></div>LIVE</div>
            <div class="spill">{{ streamStatus }}</div>
          </div>
        </div>
        <div class="mft">
          <div class="msi">
            <div class="mstrm">{{ streamModalMeta.name }}</div>
            <div v-if="streamModalMeta.isSelf" class="mvcl">{{ viewerCount }} {{ viewerCount === 1 ? 'viewer' : 'viewers' }}</div>
          </div>
          <button v-if="streamModalMeta.isSelf" class="endbtn" @click="stopLive">END STREAM</button>
        </div>
      </div>
    </div>

    <!-- SETTINGS -->
    <div v-if="showSettings" class="spnl" @click.self="showSettings = false">
      <div class="sbk" @click="showSettings = false"></div>
      <div class="sbox">
        <div class="shd"><h3>IDENTITY</h3><button class="ib" @click="showSettings = false">&times;</button></div>
        <div class="srow"><label>display name</label><input v-model="settingsName" placeholder="your name" maxlength="32" /></div>
        <div class="srow"><label>handle</label><input v-model="settingsHandle" placeholder="yourhandle" maxlength="24" /></div>
        <div class="snote">
          <strong>channels:</strong> append <code>#ch=roomname</code> to the URL.<br><br>
          <strong>relay:</strong> demo-relay.clasp.to -- CLASP router with auth. posts use SET (persisted until TTL). reactions + live signaling use emit (ephemeral). live video is WebRTC P2P.<br><br>
          <strong>images:</strong> compressed to ~60KB client-side, sent as base64 in the CLASP payload. expire with the post.
        </div>
        <button class="ssave" @click="saveSettings">SAVE</button>
      </div>
    </div>

    <!-- TOASTS -->
    <div class="toasts">
      <div v-for="t in toasts" :key="t.id" class="toast" :class="[t.type, { out: t.out }]">{{ t.msg }}</div>
    </div>
  </div>
</template>

<style scoped>
@import url('https://fonts.googleapis.com/css2?family=Oswald:wght@400;700&family=Space+Mono:wght@400;700&family=Figtree:wght@300;400;500;600&display=swap');

.social {
  --bg:#0a0a0a;--surf:#131313;--card:#161616;--card2:#1c1c1c;
  --bdr:rgba(255,255,255,0.07);--bdr2:rgba(255,255,255,0.13);
  --acc:#e85d3b;--acc-d:rgba(232,93,59,0.10);--acc-m:rgba(232,93,59,0.22);
  --red:#d44444;--red-d:rgba(212,68,68,0.10);
  --grn:#4caf72;--ylw:#d4a840;
  --txt:#a8a8a8;--br:#e8e8e8;--dim:#484848;--dim2:#242424;
  --font:'Figtree',sans-serif;--mono:'Space Mono',monospace;--disp:'Oswald',sans-serif;
  --r:6px;
  font-family: var(--font);
  font-size: 14px;
  color: var(--txt);
  min-height: 100vh;
  background: var(--bg);
}

/* TOPBAR */
.topbar{position:sticky;top:0;z-index:200;background:rgba(10,10,10,0.96);backdrop-filter:blur(24px);border-bottom:1px dashed var(--bdr);padding:0 16px;height:54px;display:flex;align-items:center;gap:10px;}
.tl{display:flex;align-items:center;gap:8px;flex-shrink:0}
.tl-word{font-family:var(--disp);font-size:18px;font-weight:700;letter-spacing:5px;color:var(--br);text-transform:uppercase}
.ch-tag{font-family:var(--mono);font-size:10px;color:var(--acc);background:var(--acc-d);border:1px solid var(--acc-m);border-radius:3px;padding:2px 7px;letter-spacing:.04em}
.tc{display:flex;align-items:center;gap:6px;flex:1;min-width:0;overflow:hidden}
.cdot{width:6px;height:6px;border-radius:50%;background:var(--dim);flex-shrink:0;transition:background .35s}
.cdot.on{background:var(--grn);animation:pdot 2.8s infinite}
.cdot.off{background:var(--red)}.cdot.rc{background:var(--ylw);animation:pdot 1s infinite}.cdot.er{background:var(--red)}
@keyframes pdot{0%,100%{box-shadow:0 0 0 0 rgba(76,175,114,.5)}60%{box-shadow:0 0 0 5px transparent}}
.conn-lbl{font-family:var(--mono);font-size:9px;color:var(--dim);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.tr{display:flex;align-items:center;gap:4px;flex-shrink:0}
.online-ct{font-family:var(--mono);font-size:10px;color:var(--dim);white-space:nowrap}
.ib{background:none;border:1px dashed transparent;color:var(--dim);cursor:pointer;width:32px;height:32px;border-radius:var(--r);display:flex;align-items:center;justify-content:center;transition:all .15s;flex-shrink:0}
.ib:hover{border-color:var(--bdr2);color:var(--br)}

/* LAYOUT */
.app{max-width:620px;margin:0 auto;padding:14px 16px 100px}

/* LIVE STRIP */
.ls{background:var(--surf);border:1px dashed var(--bdr);border-radius:var(--r);margin-bottom:12px;overflow:hidden}
.ls-hd{display:flex;align-items:center;gap:8px;padding:7px 12px;border-bottom:1px dashed var(--bdr)}
.lbdge{display:flex;align-items:center;gap:5px;background:var(--red-d);border:1px solid rgba(212,68,68,.25);border-radius:3px;padding:2px 7px;font-family:var(--mono);font-size:9px;color:var(--red);letter-spacing:.12em;text-transform:uppercase}
.lbdot{width:5px;height:5px;border-radius:50%;background:var(--red);animation:blk 1s infinite}
@keyframes blk{0%,100%{opacity:1}50%{opacity:.1}}
.live-ct{margin-left:auto;font-family:var(--mono);font-size:10px;color:var(--dim)}
.ls-sc{display:flex;gap:8px;align-items:center;padding:10px 12px;overflow-x:auto;scrollbar-width:none;min-height:44px}
.ls-sc::-webkit-scrollbar{display:none}
.ls-em{font-family:var(--mono);font-size:11px;color:var(--dim);display:flex;align-items:center;gap:6px;opacity:.7}
.lp{display:flex;align-items:center;gap:6px;background:var(--red-d);border:1px dashed rgba(212,68,68,.22);border-radius:20px;padding:5px 12px 5px 8px;font-family:var(--mono);font-size:11px;color:var(--br);cursor:pointer;flex-shrink:0;transition:all .15s;white-space:nowrap}
.lp:hover{background:rgba(212,68,68,.15);border-color:rgba(212,68,68,.4)}
.lp.own{border-color:var(--acc-m);color:var(--acc);background:var(--acc-d)}
.lp-dot{width:7px;height:7px;border-radius:50%;background:var(--red);animation:blk 1s infinite;flex-shrink:0}
.lp.own .lp-dot{background:var(--acc);animation:none}

/* FLOOD */
.flood{display:flex;align-items:center;gap:10px;background:rgba(212,168,64,.05);border:1px dashed rgba(212,168,64,.2);border-radius:var(--r);padding:9px 12px;margin-bottom:12px;font-family:var(--mono);font-size:11px;color:var(--ylw);animation:sin .3s ease}
@keyframes sin{from{opacity:0;transform:translateY(-4px)}to{opacity:1;transform:none}}
.ftk{flex:1;height:2px;background:rgba(212,168,64,.1);border-radius:2px;overflow:hidden}
.ffl{height:100%;background:var(--ylw);border-radius:2px;transition:width .4s}

/* COMPOSER */
.composer{background:var(--surf);border:1px dashed var(--bdr);border-radius:var(--r);margin-bottom:14px;transition:border-color .2s}
.composer:focus-within{border-color:var(--acc-m)}
.ct{display:flex;gap:10px;padding:13px 13px 0}
.cav{width:32px;height:32px;border-radius:50%;background:var(--acc-d);border:1px dashed var(--acc-m);display:flex;align-items:center;justify-content:center;flex-shrink:0;color:var(--acc)}
.cr{flex:1;min-width:0}
.chandl{font-family:var(--mono);font-size:10px;color:var(--acc);margin-bottom:5px}
.cr textarea{width:100%;background:transparent;border:none;outline:none;color:var(--br);font-family:var(--font);font-size:14px;line-height:1.65;resize:none;min-height:52px;max-height:260px;overflow-y:auto}
.cr textarea::placeholder{color:var(--dim)}
.img-prev{margin:10px 13px 0;border-radius:var(--r);overflow:hidden;border:1px dashed var(--bdr);background:var(--bg);max-height:180px;position:relative}
.img-prev img{width:100%;max-height:180px;object-fit:cover;display:block}
.img-prev-rm{position:absolute;top:6px;right:6px;background:rgba(10,10,10,.82);border:none;color:var(--br);border-radius:4px;width:26px;height:26px;display:flex;align-items:center;justify-content:center;cursor:pointer;font-size:16px}
.img-prev-rm:hover{background:var(--red)}
.url-row{display:flex;align-items:center;gap:6px;padding:8px 13px 0}
.url-row input{flex:1;background:var(--card);border:1px dashed var(--bdr);border-radius:4px;color:var(--txt);font-family:var(--mono);font-size:11px;padding:5px 9px;outline:none}
.url-row input:focus{border-color:var(--acc-m)}
.ttl-row{display:flex;align-items:center;gap:4px;padding:8px 13px 0}
.ttl-lbl{font-family:var(--mono);font-size:10px;color:var(--dim);margin-right:2px;flex-shrink:0}
.ttb{background:none;border:1px dashed var(--bdr);color:var(--dim);border-radius:3px;padding:2px 7px;font-family:var(--mono);font-size:10px;cursor:pointer;transition:all .15s}
.ttb.on{background:var(--acc-d);border-color:var(--acc-m);color:var(--acc)}
.ttb:hover:not(.on){border-color:var(--dim2);color:var(--txt)}
.ca{display:flex;align-items:center;padding:9px 13px 12px;border-top:1px dashed var(--bdr);gap:4px;margin-top:9px}
.ctool{background:none;border:none;cursor:pointer;color:var(--dim);padding:5px 7px;border-radius:5px;display:flex;align-items:center;gap:5px;transition:all .15s;font-family:var(--mono);font-size:10px;letter-spacing:.03em}
.ctool:hover{background:var(--card2);color:var(--br)}
.cc{font-family:var(--mono);font-size:10px;color:var(--dim);margin-left:auto;user-select:none}
.cc.wrn{color:var(--ylw)}.cc.ovr{color:var(--red)}
.lbtn{display:flex;align-items:center;gap:6px;background:none;border:1px dashed rgba(212,68,68,.3);color:var(--red);border-radius:5px;padding:6px 11px;font-family:var(--mono);font-size:10px;cursor:pointer;transition:all .15s;letter-spacing:.04em}
.lbtn:hover{background:var(--red-d);border-color:var(--red)}
.lbtn.on{background:var(--red);color:#fff;border-color:var(--red);animation:lpulse 2s infinite}
@keyframes lpulse{0%,100%{box-shadow:0 0 0 0 rgba(212,68,68,.4)}50%{box-shadow:0 0 0 6px transparent}}
.txbtn{background:var(--acc);color:var(--bg);border:none;border-radius:5px;padding:7px 16px;font-family:var(--disp);font-size:11px;font-weight:700;letter-spacing:3px;text-transform:uppercase;cursor:pointer;transition:all .15s;display:flex;align-items:center;gap:6px}
.txbtn:hover{filter:brightness(1.1);transform:translateY(-1px)}

/* FEED HEADER */
.fhd{display:flex;align-items:center;gap:10px;margin-bottom:12px}
.flbl{font-family:var(--mono);font-size:10px;color:var(--dim);text-transform:uppercase;letter-spacing:.1em;white-space:nowrap}
.fln{flex:1;height:1px;background:var(--bdr)}
.fct{font-family:var(--mono);font-size:10px;color:var(--dim);white-space:nowrap}

/* QUEUED */
.qi{display:flex;align-items:center;gap:8px;padding:6px 10px;background:rgba(212,168,64,.04);border:1px dashed rgba(212,168,64,.12);border-radius:var(--r);margin-bottom:6px;font-family:var(--mono);font-size:11px;color:var(--ylw);animation:sin .3s ease}
.qdot{width:5px;height:5px;border-radius:50%;background:var(--ylw);animation:blk 1.5s infinite;flex-shrink:0}
.qt{margin-left:auto;font-size:9px;opacity:.4}

/* POSTS */
.post{background:var(--surf);border:1px dashed var(--bdr);border-radius:var(--r);margin-bottom:7px;overflow:hidden;transition:border-color .2s;animation:pin2 .3s cubic-bezier(.34,1.56,.64,1)}
@keyframes pin2{from{opacity:0;transform:translateY(7px) scale(.99)}to{opacity:1;transform:none}}
.post:hover{border-color:var(--bdr2)}
.post.exp{border-color:rgba(212,68,68,.18)}
.ph{display:flex;align-items:center;gap:9px;padding:11px 12px 8px}
.pav{width:29px;height:29px;border-radius:50%;background:var(--card2);border:1px dashed var(--bdr);display:flex;align-items:center;justify-content:center;flex-shrink:0;color:var(--acc)}
.pm{flex:1;min-width:0}
.pn{font-size:13px;font-weight:600;color:var(--br);overflow:hidden;text-overflow:ellipsis;white-space:nowrap;line-height:1.2}
.phan{font-family:var(--mono);font-size:10px;color:var(--dim);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.pr{display:flex;flex-direction:column;align-items:flex-end;gap:3px;flex-shrink:0}
.stag{font-family:var(--mono);font-size:9px;border-radius:2px;padding:1px 5px;letter-spacing:.08em;text-transform:uppercase;background:var(--card2);border:1px dashed var(--bdr);color:var(--dim)}
.stag.set{color:var(--acc);background:var(--acc-d);border-color:var(--acc-m)}
.stag.stream{color:var(--red);background:var(--red-d);border-color:rgba(212,68,68,.2)}
.stag.event{color:var(--ylw);background:rgba(212,168,64,.08);border-color:rgba(212,168,64,.2)}
.pt{font-family:var(--mono);font-size:10px;color:var(--dim)}
.pb{padding:0 12px 9px;font-size:14px;line-height:1.65;color:var(--br);word-break:break-word;white-space:pre-wrap}
.pimg{margin:0 12px 9px;border-radius:5px;overflow:hidden;border:1px dashed var(--bdr);background:var(--bg)}
.pimg img{width:100%;max-height:280px;object-fit:cover;display:block}
.pf{display:flex;align-items:center;padding:5px 10px 8px;gap:1px;border-top:1px dashed var(--bdr)}
.ab{background:none;border:none;cursor:pointer;color:var(--dim);padding:4px 8px;border-radius:4px;display:flex;align-items:center;gap:5px;transition:all .15s;font-family:var(--mono);font-size:11px}
.ab:hover{background:var(--card2);color:var(--br)}
.rb.act{color:var(--acc)}.rb.act svg{fill:var(--acc)}
.rc{font-size:10px}
.db{margin-left:auto}.db:hover{color:var(--red);background:rgba(212,68,68,.06)}
.pex{height:2px;background:var(--card2)}
.pexf{height:100%;background:var(--acc);transition:width 1.5s linear}

.fempty{text-align:center;padding:60px 20px;font-family:var(--mono);font-size:12px;color:var(--dim);line-height:1.9}

/* STREAM MODAL */
.modal{position:fixed;inset:0;z-index:500;display:flex;align-items:center;justify-content:center;padding:16px;animation:mfad .2s ease}
@keyframes mfad{from{opacity:0}to{opacity:1}}
.mbk{position:absolute;inset:0;background:rgba(0,0,0,.88);backdrop-filter:blur(16px)}
.mbox{position:relative;z-index:1;background:var(--surf);border:1px dashed var(--bdr2);border-radius:var(--r);width:min(560px,100%);max-height:90vh;overflow:hidden;animation:mmin .25s cubic-bezier(.34,1.56,.64,1)}
@keyframes mmin{from{transform:scale(.94) translateY(10px)}to{transform:none}}
.mhd{display:flex;align-items:center;gap:10px;padding:12px 15px;border-bottom:1px dashed var(--bdr)}
.mttl{flex:1;min-width:0}
.mttl-name{font-family:var(--disp);font-size:13px;font-weight:700;letter-spacing:3px;text-transform:uppercase;color:var(--br);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.msub{font-family:var(--mono);font-size:10px;color:var(--dim);margin-top:2px}
.mbdge{display:flex;align-items:center;gap:4px;background:var(--red-d);border:1px solid rgba(212,68,68,.25);border-radius:3px;padding:2px 7px;font-family:var(--mono);font-size:9px;color:var(--red);letter-spacing:.12em;text-transform:uppercase;flex-shrink:0}
.mbdot{width:5px;height:5px;border-radius:50%;background:var(--red);animation:blk 1s infinite}
.mclose{background:none;border:1px dashed transparent;cursor:pointer;color:var(--dim);width:30px;height:30px;border-radius:5px;display:flex;align-items:center;justify-content:center;transition:all .15s;flex-shrink:0;font-size:18px}
.mclose:hover{border-color:var(--bdr2);color:var(--br)}
.mvw{position:relative;background:#000;aspect-ratio:16/9;overflow:hidden}
.mvw video{width:100%;height:100%;object-fit:cover}
.shud{position:absolute;top:9px;left:9px;display:flex;gap:6px;align-items:center}
.slive{background:var(--red);color:#fff;font-family:var(--mono);font-size:9px;padding:2px 7px;border-radius:3px;letter-spacing:.1em;display:flex;align-items:center;gap:4px}
.sbdot{width:5px;height:5px;border-radius:50%;background:#fff;animation:blk 1s infinite}
.spill{background:rgba(0,0,0,.68);color:var(--txt);font-family:var(--mono);font-size:9px;padding:2px 8px;border-radius:3px}
.mft{display:flex;align-items:center;gap:12px;padding:11px 15px;border-top:1px dashed var(--bdr)}
.msi{flex:1}
.mstrm{font-size:13px;font-weight:600;color:var(--br)}
.mvcl{font-family:var(--mono);font-size:10px;color:var(--dim);margin-top:2px}
.endbtn{background:var(--red-d);border:1px dashed rgba(212,68,68,.3);color:var(--red);border-radius:5px;padding:7px 13px;font-family:var(--mono);font-size:10px;cursor:pointer;transition:all .15s;display:flex;align-items:center;gap:6px;flex-shrink:0}
.endbtn:hover{background:var(--red);color:#fff;border-color:var(--red)}

/* SETTINGS */
.spnl{position:fixed;inset:0;z-index:400;display:flex;align-items:flex-end;justify-content:center;animation:mfad .2s ease}
.sbk{position:absolute;inset:0;background:rgba(0,0,0,.75);backdrop-filter:blur(10px)}
.sbox{position:relative;z-index:1;background:var(--surf);border:1px dashed var(--bdr2);border-top-left-radius:10px;border-top-right-radius:10px;width:min(620px,100vw);padding:20px 20px max(32px,env(safe-area-inset-bottom));animation:sup .25s cubic-bezier(.34,1.56,.64,1)}
@keyframes sup{from{transform:translateY(100%)}to{transform:none}}
.shd{display:flex;align-items:center;margin-bottom:20px}
.shd h3{font-family:var(--disp);font-size:13px;font-weight:700;letter-spacing:3px;text-transform:uppercase;color:var(--br);flex:1}
.srow{margin-bottom:14px}
.srow label{display:block;font-family:var(--mono);font-size:10px;color:var(--dim);text-transform:uppercase;letter-spacing:.1em;margin-bottom:6px}
.srow input{width:100%;background:var(--card);border:1px dashed var(--bdr);border-radius:4px;color:var(--br);font-family:var(--mono);font-size:13px;padding:9px 11px;outline:none;transition:border-color .2s}
.srow input:focus{border-color:var(--acc-m)}
.snote{font-family:var(--mono);font-size:10px;color:var(--dim);line-height:1.8;margin-bottom:16px;border-top:1px dashed var(--bdr);padding-top:14px}
.snote strong{color:var(--txt)}
.ssave{background:var(--acc);color:var(--bg);border:none;border-radius:5px;padding:9px 20px;font-family:var(--disp);font-size:11px;font-weight:700;letter-spacing:3px;text-transform:uppercase;cursor:pointer;transition:filter .15s}
.ssave:hover{filter:brightness(1.1)}

/* NAME PICKER */
.npick{position:fixed;inset:0;z-index:1000;display:flex;align-items:center;justify-content:center;background:rgba(10,10,10,0.97);backdrop-filter:blur(20px);padding:20px}
.npbox{background:var(--surf);border:1px dashed var(--bdr2);border-radius:var(--r);padding:32px 28px;width:min(380px,100%);animation:mmin .4s cubic-bezier(.34,1.56,.64,1)}
.npttl{font-family:var(--disp);font-size:18px;font-weight:700;letter-spacing:4px;text-transform:uppercase;color:var(--br);text-align:center;margin-bottom:6px}
.npsub{font-family:var(--mono);font-size:11px;color:var(--dim);text-align:center;margin-bottom:22px;line-height:1.7}
.nprow{display:flex;flex-direction:column;gap:6px;margin-bottom:14px}
.nprow label{font-family:var(--mono);font-size:10px;color:var(--dim);text-transform:uppercase;letter-spacing:.1em}
.nprow input{width:100%;background:var(--card);border:1px dashed var(--bdr);border-radius:4px;color:var(--br);font-family:var(--mono);font-size:14px;padding:10px 12px;outline:none;transition:border-color .2s}
.nprow input:focus{border-color:var(--acc-m)}
.npbtn{width:100%;background:var(--acc);color:var(--bg);border:none;border-radius:5px;padding:11px;font-family:var(--disp);font-size:12px;font-weight:700;letter-spacing:4px;text-transform:uppercase;cursor:pointer;transition:filter .15s;margin-top:4px}
.npbtn:hover{filter:brightness(1.1)}
.npskip{display:block;text-align:center;font-family:var(--mono);font-size:10px;color:var(--dim);cursor:pointer;margin-top:10px;background:none;border:none;width:100%;transition:color .15s}
.npskip:hover{color:var(--txt)}

/* TOASTS */
.toasts{position:fixed;bottom:20px;right:20px;z-index:600;display:flex;flex-direction:column;gap:6px;pointer-events:none}
.toast{background:var(--card);border:1px dashed var(--bdr2);border-radius:5px;padding:9px 13px;font-family:var(--mono);font-size:11px;color:var(--br);max-width:280px;min-width:160px;animation:tin .25s ease;pointer-events:auto}
.toast.wrn{border-color:rgba(212,168,64,.3);color:var(--ylw)}
.toast.err{border-color:rgba(212,68,68,.3);color:var(--red)}
@keyframes tin{from{opacity:0;transform:translateX(12px)}to{opacity:1;transform:none}}
.toast.out{animation:tout .25s ease forwards}
@keyframes tout{to{opacity:0;transform:translateX(12px)}}

@media(max-width:480px){
  .conn-lbl{display:none}.topbar{padding:0 12px}.app{padding:10px 12px 80px}
  .tl-word{font-size:16px;letter-spacing:4px}
  .modal{align-items:flex-end;padding:0}.mbox{width:100%;border-radius:8px 8px 0 0}
}
</style>
