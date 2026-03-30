<script setup>
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { useRelay } from '../composables/useRelay.js'

const emit = defineEmits(['auth-required'])
const { client, connected, userName, authToken, connect } = useRelay()

// --- Identity ---
const myId = ref(localStorage.getItem('clasp_spaces_id') || crypto.randomUUID().replace(/-/g, '').slice(0, 12))
const myName = ref('')
const myColor = ref('#' + myId.value.slice(0, 6).replace(/[^0-9a-f]/gi, 'a'))
localStorage.setItem('clasp_spaces_id', myId.value)

const PREFIX = '/audiospace'
const DIR = `${PREFIX}/directory`

// --- State ---
const view = ref('lobby') // 'lobby' | 'room'
const roomDirectory = reactive({})
const currentRoom = ref(null)
const myRole = ref('listener')
const isMuted = ref(true)
const handUp = ref(false)
const chatOpen = ref(false)
const chatMsg = ref('')
const chatMessages = ref([])
const participants = reactive({})
const speakingState = reactive({})
const handState = reactive({})
const searchQuery = ref('')
const showCreate = ref(false)
const newRoomName = ref('')
const newRoomDesc = ref('')

const roomIcons = ['*', '#', '~', '+', '^', '&', '@', '%']
const selectedIcon = ref('*')

const unsubs = []
let presenceInterval = null
let localStream = null
const peers = {}

const ICE_CFG = { iceServers: [{ urls: 'stun:stun.l.google.com:19302' }] }

// --- Computed ---
const filteredRooms = computed(() => {
  const q = searchQuery.value.toLowerCase()
  return Object.values(roomDirectory)
    .filter(r => !r.__ended)
    .filter(r => !q || r.name?.toLowerCase().includes(q) || r.desc?.toLowerCase().includes(q))
    .sort((a, b) => (b.count || 0) - (a.count || 0))
})

const speakers = computed(() => Object.values(participants).filter(p => !p.__left && (p.role === 'speaker' || p.isHost)))
const listeners = computed(() => Object.values(participants).filter(p => !p.__left && p.role === 'listener'))
const canSpeak = computed(() => myRole.value === 'host' || myRole.value === 'speaker')

// --- Directory subscriptions ---
function setupDirectory() {
  const c = client.value
  if (!c) return
  c.on(`${DIR}/**`, (v, addr) => {
    const id = addr.split('/').pop()
    if (!v || v.__ended) { delete roomDirectory[id]; return }
    roomDirectory[id] = v
  })
}

// --- Room management ---
function createRoom() {
  const name = newRoomName.value.trim()
  if (!name) return
  const c = client.value
  if (!c) return

  const id = crypto.randomUUID().replace(/-/g, '').slice(0, 10)
  const meta = {
    id, name, desc: newRoomDesc.value.trim(),
    icon: selectedIcon.value,
    hostId: myId.value, hostName: myName.value,
    createdAt: Date.now(), count: 1,
  }
  c.set(`${DIR}/${id}`, meta)
  showCreate.value = false
  newRoomName.value = ''
  newRoomDesc.value = ''
  joinRoom(id, meta)
}

function joinRoom(roomId, metaOverride) {
  const c = client.value
  if (!c) return
  const meta = metaOverride || roomDirectory[roomId]
  if (!meta) return

  currentRoom.value = meta
  myRole.value = meta.hostId === myId.value ? 'host' : 'listener'
  isMuted.value = true
  handUp.value = false
  chatOpen.value = false
  chatMessages.value = []
  Object.keys(participants).forEach(k => delete participants[k])
  Object.keys(speakingState).forEach(k => delete speakingState[k])
  Object.keys(handState).forEach(k => delete handState[k])
  cleanupRoom()

  view.value = 'room'
  const base = `${PREFIX}/rooms/${roomId}`

  unsubs.push(c.on(`${base}/participants/**`, (v, a) => {
    const uid = a.split('/').pop()
    if (!v || v.__left) { delete participants[uid]; destroyPeer(uid) }
    else {
      const isNew = !participants[uid]
      participants[uid] = v
      if (isNew && uid !== myId.value && uid > myId.value) createPC(uid, true)
    }
    updateDirCount()
  }))

  unsubs.push(c.on(`${base}/speaking/**`, (v, a) => {
    const uid = a.split('/').pop()
    if (v) speakingState[uid] = v
  }))

  unsubs.push(c.on(`${base}/hands/**`, (v, a) => {
    handState[a.split('/').pop()] = v
  }))

  unsubs.push(c.on(`${base}/chat/**`, (v) => {
    if (v?.text) chatMessages.value.push(v)
  }))

  unsubs.push(c.on(`${base}/signal/${myId.value}/**`, (v, a) => {
    if (v) handleSignal(a.split('/').pop(), v)
  }))

  unsubs.push(c.on(`${base}/roles/${myId.value}`, (v) => {
    if (!v || myRole.value === 'host') return
    if (v.role === 'speaker' && myRole.value !== 'speaker') {
      myRole.value = 'speaker'
      announcePresence()
    } else if (v.role === 'listener' && myRole.value === 'speaker') {
      myRole.value = 'listener'
      if (!isMuted.value) { stopMic(); isMuted.value = true }
      announcePresence()
    }
  }))

  unsubs.push(c.on(`${base}/ended`, (v) => {
    if (v?.ended && myRole.value !== 'host') {
      leaveRoom()
    }
  }))

  announcePresence()
  presenceInterval = setInterval(announcePresence, 6000)
}

function announcePresence() {
  const c = client.value
  if (!c || !currentRoom.value) return
  c.set(`${PREFIX}/rooms/${currentRoom.value.id}/participants/${myId.value}`, {
    id: myId.value, name: myName.value,
    role: myRole.value === 'host' ? 'speaker' : myRole.value,
    color: myColor.value, isHost: myRole.value === 'host',
    ts: Date.now(),
  })
}

function updateDirCount() {
  const c = client.value
  if (!c || !currentRoom.value) return
  const count = Object.values(participants).filter(p => !p.__left).length
  c.set(`${DIR}/${currentRoom.value.id}`, { ...roomDirectory[currentRoom.value.id], ...currentRoom.value, count })
}

function leaveRoom() {
  const c = client.value
  if (c && currentRoom.value) {
    c.set(`${PREFIX}/rooms/${currentRoom.value.id}/participants/${myId.value}`, { __left: true, ts: Date.now() })
    c.set(`${PREFIX}/rooms/${currentRoom.value.id}/speaking/${myId.value}`, null)
    c.set(`${PREFIX}/rooms/${currentRoom.value.id}/hands/${myId.value}`, null)
  }
  cleanupRoom()
  stopMic()
  currentRoom.value = null
  myRole.value = 'listener'
  view.value = 'lobby'
}

function endRoom() {
  if (!currentRoom.value || myRole.value !== 'host') return
  const c = client.value
  if (c) {
    c.set(`${PREFIX}/rooms/${currentRoom.value.id}/ended`, { ended: true, ts: Date.now() })
    c.set(`${DIR}/${currentRoom.value.id}`, { __ended: true, ts: Date.now() })
  }
  leaveRoom()
}

function cleanupRoom() {
  unsubs.forEach(u => { try { u() } catch {} })
  unsubs.length = 0
  if (presenceInterval) { clearInterval(presenceInterval); presenceInterval = null }
  Object.keys(peers).forEach(destroyPeer)
}

// --- Host controls ---
function promote(uid) {
  const c = client.value
  if (myRole.value !== 'host' || !c || !currentRoom.value) return
  c.set(`${PREFIX}/rooms/${currentRoom.value.id}/roles/${uid}`, { role: 'speaker', ts: Date.now() })
  c.set(`${PREFIX}/rooms/${currentRoom.value.id}/hands/${uid}`, null)
}

function demote(uid) {
  const c = client.value
  if (myRole.value !== 'host' || !c || !currentRoom.value || uid === myId.value) return
  c.set(`${PREFIX}/rooms/${currentRoom.value.id}/roles/${uid}`, { role: 'listener', ts: Date.now() })
}

// --- Mic ---
async function toggleMic() {
  if (!canSpeak.value) return
  if (isMuted.value) {
    try {
      localStream = await navigator.mediaDevices.getUserMedia({ audio: true })
      isMuted.value = false
      // Add tracks to existing peers
      Object.values(peers).forEach(p => {
        if (p.pc && localStream) {
          localStream.getTracks().forEach(t => p.pc.addTrack(t, localStream))
        }
      })
      startSpeakingDetection()
    } catch (e) {
      console.error('[mic]', e)
    }
  } else {
    stopMic()
    isMuted.value = true
  }
}

function stopMic() {
  if (localStream) {
    localStream.getTracks().forEach(t => t.stop())
    localStream = null
  }
  stopSpeakingDetection()
}

// --- Speaking detection ---
let analyserTimer = null
let analyserCtx = null
function startSpeakingDetection() {
  if (!localStream) return
  analyserCtx = new AudioContext()
  const src = analyserCtx.createMediaStreamSource(localStream)
  const analyser = analyserCtx.createAnalyser()
  analyser.fftSize = 256
  src.connect(analyser)
  const data = new Uint8Array(analyser.frequencyBinCount)
  let wasSpeaking = false

  analyserTimer = setInterval(() => {
    analyser.getByteFrequencyData(data)
    const avg = data.reduce((a, b) => a + b, 0) / data.length
    const speaking = avg > 15
    if (speaking !== wasSpeaking) {
      wasSpeaking = speaking
      const c = client.value
      if (c && currentRoom.value) {
        c.stream(`${PREFIX}/rooms/${currentRoom.value.id}/speaking/${myId.value}`, { speaking, vol: avg })
      }
    }
  }, 100)
}

function stopSpeakingDetection() {
  if (analyserTimer) { clearInterval(analyserTimer); analyserTimer = null }
  if (analyserCtx) { analyserCtx.close().catch(() => {}); analyserCtx = null }
  const c = client.value
  if (c && currentRoom.value) {
    c.set(`${PREFIX}/rooms/${currentRoom.value.id}/speaking/${myId.value}`, null)
  }
}

function toggleHand() {
  const c = client.value
  if (!c || !currentRoom.value) return
  handUp.value = !handUp.value
  c.set(`${PREFIX}/rooms/${currentRoom.value.id}/hands/${myId.value}`, handUp.value ? { ts: Date.now() } : null)
}

function sendChat() {
  const text = chatMsg.value.trim()
  if (!text) return
  const c = client.value
  if (!c || !currentRoom.value) return
  const id = crypto.randomUUID().replace(/-/g, '').slice(0, 8)
  c.emit(`${PREFIX}/rooms/${currentRoom.value.id}/chat/${id}`, {
    text, name: myName.value, id: myId.value, ts: Date.now(),
  })
  chatMessages.value.push({ text, name: myName.value, id: myId.value, ts: Date.now() })
  chatMsg.value = ''
}

// --- WebRTC ---
async function createPC(pid, init) {
  if (peers[pid]) return
  const pc = new RTCPeerConnection(ICE_CFG)
  const el = document.createElement('audio')
  el.autoplay = true
  el.playsInline = true
  document.body.appendChild(el)
  peers[pid] = { pc, audioEl: el }

  if (localStream) localStream.getTracks().forEach(t => pc.addTrack(t, localStream))

  pc.ontrack = (ev) => { el.srcObject = ev.streams[0] }
  pc.onicecandidate = (ev) => {
    if (!ev.candidate) return
    const c = client.value
    if (c && currentRoom.value) {
      c.set(`${PREFIX}/rooms/${currentRoom.value.id}/signal/${pid}/${myId.value}`, {
        type: 'ice', candidate: ev.candidate.toJSON(), ts: Date.now(),
      })
    }
  }

  if (init) {
    if (!localStream) pc.addTransceiver('audio', { direction: 'recvonly' })
    const offer = await pc.createOffer()
    await pc.setLocalDescription(offer)
    const c = client.value
    if (c && currentRoom.value) {
      c.set(`${PREFIX}/rooms/${currentRoom.value.id}/signal/${pid}/${myId.value}`, {
        type: 'offer', sdp: offer.sdp, ts: Date.now(),
      })
    }
  }
}

async function handleSignal(from, d) {
  const c = client.value
  if (d.type === 'offer') {
    if (!peers[from]) await createPC(from, false)
    const pc = peers[from].pc
    await pc.setRemoteDescription(new RTCSessionDescription({ type: 'offer', sdp: d.sdp }))
    if (localStream && !pc.getSenders().length) {
      localStream.getTracks().forEach(t => pc.addTrack(t, localStream))
    }
    const ans = await pc.createAnswer()
    await pc.setLocalDescription(ans)
    if (c && currentRoom.value) {
      c.set(`${PREFIX}/rooms/${currentRoom.value.id}/signal/${from}/${myId.value}`, {
        type: 'answer', sdp: ans.sdp, ts: Date.now(),
      })
    }
  } else if (d.type === 'answer') {
    if (peers[from]?.pc.signalingState === 'have-local-offer') {
      await peers[from].pc.setRemoteDescription(new RTCSessionDescription({ type: 'answer', sdp: d.sdp }))
    }
  } else if (d.type === 'ice' && peers[from] && d.candidate) {
    try { await peers[from].pc.addIceCandidate(new RTCIceCandidate(d.candidate)) } catch {}
  }
}

function destroyPeer(pid) {
  if (!peers[pid]) return
  peers[pid].pc.close()
  peers[pid].audioEl.remove()
  delete peers[pid]
}

// --- Lifecycle ---
onMounted(async () => {
  if (!authToken.value) { emit('auth-required'); return }
  myName.value = userName.value || 'Anon'
  try {
    await connect()
    setupDirectory()
  } catch (e) { console.error('[Spaces]', e) }
})

onUnmounted(() => {
  if (currentRoom.value) leaveRoom()
  stopMic()
})
</script>

<template>
  <div class="spaces">
    <div class="container">
      <!-- Lobby -->
      <template v-if="view === 'lobby'">
        <div class="page-head">
          <h1 class="page-title">AUDIO SPACES</h1>
          <button class="create-btn" @click="showCreate = true">+ New Room</button>
        </div>

        <div class="search-bar">
          <input v-model="searchQuery" placeholder="Search rooms..." />
        </div>

        <div class="room-list">
          <div
            v-for="room in filteredRooms"
            :key="room.id"
            class="room-card"
            @click="joinRoom(room.id)"
          >
            <div class="room-icon">{{ room.icon || '*' }}</div>
            <div class="room-info">
              <div class="room-name">{{ room.name }}</div>
              <div v-if="room.desc" class="room-desc">{{ room.desc }}</div>
              <div class="room-meta-line">
                <span class="room-host">{{ room.hostName }}</span>
                <span class="room-count">{{ room.count || 0 }} in room</span>
              </div>
            </div>
          </div>
          <div v-if="!filteredRooms.length" class="empty-rooms">
            No rooms yet. Create one to get started.
          </div>
        </div>
      </template>

      <!-- Room -->
      <template v-if="view === 'room' && currentRoom">
        <div class="room-header">
          <button class="back-btn" @click="leaveRoom">&larr;</button>
          <div class="room-header-info">
            <span class="room-header-icon">{{ currentRoom.icon }}</span>
            <span class="room-header-name">{{ currentRoom.name }}</span>
          </div>
          <span v-if="myRole === 'host'" class="host-badge">HOST</span>
        </div>

        <!-- Speakers -->
        <div class="section-label">Speakers</div>
        <div class="participant-grid">
          <div
            v-for="p in speakers"
            :key="p.id"
            class="participant"
            :class="{ speaking: speakingState[p.id]?.speaking }"
          >
            <div class="p-avatar" :style="{ borderColor: p.color || '#555' }">
              {{ p.name?.[0]?.toUpperCase() }}
            </div>
            <div class="p-name">{{ p.name }}</div>
            <div v-if="p.isHost" class="p-host-tag">host</div>
            <button
              v-if="myRole === 'host' && !p.isHost"
              class="p-action demote"
              @click.stop="demote(p.id)"
              title="Move to listeners"
            >v</button>
          </div>
        </div>

        <!-- Listeners -->
        <div class="section-label">
          Listeners
          <span class="listener-ct">{{ listeners.length }}</span>
        </div>
        <div class="participant-grid">
          <div
            v-for="p in listeners"
            :key="p.id"
            class="participant"
          >
            <div class="p-avatar" :style="{ borderColor: p.color || '#555' }">
              {{ p.name?.[0]?.toUpperCase() }}
            </div>
            <div class="p-name">{{ p.name }}</div>
            <span v-if="handState[p.id]" class="hand-icon" title="Wants to speak">^</span>
            <button
              v-if="myRole === 'host' && handState[p.id]"
              class="p-action promote"
              @click.stop="promote(p.id)"
              title="Invite to speak"
            >^</button>
          </div>
        </div>

        <!-- Chat panel -->
        <div v-if="chatOpen" class="chat-panel fade-in">
          <div class="chat-msgs">
            <div v-for="(m, i) in chatMessages" :key="i" class="chat-msg">
              <span class="chat-name">{{ m.name }}</span>
              <span class="chat-text">{{ m.text }}</span>
            </div>
          </div>
          <form class="chat-input" @submit.prevent="sendChat">
            <input v-model="chatMsg" placeholder="Message..." />
            <button type="submit">Send</button>
          </form>
        </div>

        <!-- Controls bar -->
        <div class="controls-bar">
          <button class="ctrl-btn" :class="{ active: chatOpen }" @click="chatOpen = !chatOpen" title="Chat">
            Chat
          </button>
          <button
            v-if="myRole === 'listener'"
            class="ctrl-btn"
            :class="{ active: handUp }"
            @click="toggleHand"
            title="Request to speak"
          >
            Hand {{ handUp ? '(up)' : '' }}
          </button>
          <button
            v-if="canSpeak"
            class="mic-btn"
            :class="{ on: !isMuted }"
            @click="toggleMic"
          >
            {{ isMuted ? 'Unmute' : 'Mute' }}
          </button>
          <button v-if="myRole === 'host'" class="ctrl-btn end" @click="endRoom">
            End
          </button>
          <button class="ctrl-btn leave" @click="leaveRoom">Leave</button>
        </div>
      </template>
    </div>

    <!-- Create modal -->
    <div v-if="showCreate" class="overlay" @click.self="showCreate = false">
      <div class="modal fade-in">
        <div class="modal-head">
          <span class="modal-title">Create Room</span>
          <button class="modal-close" @click="showCreate = false">&times;</button>
        </div>
        <form class="modal-body" @submit.prevent="createRoom">
          <div class="icon-picker">
            <button
              v-for="ic in roomIcons"
              :key="ic"
              type="button"
              class="icon-opt"
              :class="{ selected: selectedIcon === ic }"
              @click="selectedIcon = ic"
            >{{ ic }}</button>
          </div>
          <label>
            <span class="lbl">Room name</span>
            <input v-model="newRoomName" placeholder="My room" required />
          </label>
          <label>
            <span class="lbl">Description (optional)</span>
            <input v-model="newRoomDesc" placeholder="What's this room about?" />
          </label>
          <button type="submit" class="btn-primary">Create Room</button>
        </form>
      </div>
    </div>
  </div>
</template>

<style scoped>
.spaces { min-height: 100vh; padding-bottom: 100px; }
.page-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 0 16px;
  border-bottom: 1px solid var(--bdr);
  margin-bottom: 16px;
}
.page-title {
  font-family: var(--head);
  font-size: 16px;
  letter-spacing: 0.15em;
  color: var(--br);
}
.create-btn {
  font-family: var(--mono);
  font-size: 11px;
  color: var(--teal);
  border: 1px solid var(--teal-m);
  padding: 6px 14px;
  border-radius: var(--r);
  transition: background 0.15s;
}
.create-btn:hover { background: var(--teal-d); }
.search-bar { margin-bottom: 14px; }
.search-bar input { width: 100%; }

/* Room list */
.room-list { display: flex; flex-direction: column; gap: 8px; }
.room-card {
  display: flex;
  gap: 12px;
  padding: 14px;
  background: var(--surf);
  border: 1px solid var(--bdr);
  border-radius: 6px;
  cursor: pointer;
  transition: border-color 0.2s;
}
.room-card:hover { border-color: var(--teal-m); }
.room-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  background: var(--dim2);
  border-radius: 8px;
  flex-shrink: 0;
  color: var(--teal);
}
.room-info { flex: 1; min-width: 0; }
.room-name { font-weight: 700; color: var(--br); font-size: 13px; margin-bottom: 2px; }
.room-desc { font-size: 11px; color: var(--dim); margin-bottom: 4px; }
.room-meta-line { display: flex; gap: 10px; font-size: 10px; }
.room-host { color: var(--teal); font-family: var(--mono); }
.room-count { color: var(--dim); font-family: var(--mono); }
.empty-rooms { text-align: center; padding: 48px 16px; color: var(--dim); font-size: 13px; }

/* Room view */
.room-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 16px 0;
  border-bottom: 1px solid var(--bdr);
  margin-bottom: 16px;
}
.back-btn { font-size: 18px; color: var(--dim); padding: 4px 8px; }
.back-btn:hover { color: var(--br); }
.room-header-info { display: flex; align-items: center; gap: 8px; flex: 1; }
.room-header-icon { font-size: 18px; color: var(--teal); }
.room-header-name { font-family: var(--head); font-size: 14px; color: var(--br); letter-spacing: 0.08em; }
.host-badge {
  font-family: var(--mono);
  font-size: 9px;
  color: var(--teal);
  background: var(--teal-d);
  border: 1px solid var(--teal-m);
  padding: 2px 8px;
  border-radius: 3px;
  letter-spacing: 0.1em;
}

.section-label {
  font-family: var(--mono);
  font-size: 10px;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--dim);
  margin-bottom: 10px;
  display: flex;
  align-items: center;
  gap: 8px;
}
.listener-ct { color: var(--dim); }

.participant-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-bottom: 20px;
}
.participant {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  width: 72px;
  position: relative;
}
.participant.speaking .p-avatar {
  box-shadow: 0 0 0 3px var(--teal);
}
.p-avatar {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: var(--dim2);
  border: 2px solid var(--bdr2);
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--head);
  font-size: 16px;
  color: var(--br);
  transition: box-shadow 0.2s;
}
.p-name {
  font-size: 10px;
  color: var(--dim);
  text-align: center;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 72px;
}
.p-host-tag {
  font-family: var(--mono);
  font-size: 8px;
  color: var(--teal);
}
.hand-icon { color: var(--ylw); font-size: 12px; }
.p-action {
  position: absolute;
  top: -4px;
  right: -4px;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  font-size: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.p-action.promote { background: var(--teal-d); color: var(--teal); border: 1px solid var(--teal-m); }
.p-action.demote { background: var(--red-d); color: var(--red); border: 1px solid rgba(230,57,70,0.3); }

/* Chat */
.chat-panel {
  background: var(--surf);
  border: 1px solid var(--bdr);
  border-radius: 6px;
  margin-bottom: 16px;
  max-height: 300px;
  display: flex;
  flex-direction: column;
}
.chat-msgs {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: 240px;
}
.chat-msg { font-size: 12px; }
.chat-name { color: var(--teal); font-weight: 700; margin-right: 6px; }
.chat-text { color: var(--br); }
.chat-input {
  display: flex;
  gap: 8px;
  padding: 8px 10px;
  border-top: 1px solid var(--bdr);
}
.chat-input input { flex: 1; font-size: 12px; padding: 6px 10px; }
.chat-input button {
  font-size: 11px;
  color: var(--teal);
  padding: 6px 12px;
  border: 1px solid var(--teal-m);
  border-radius: var(--r);
}

/* Controls */
.controls-bar {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px 16px;
  background: rgba(10,10,10,0.95);
  backdrop-filter: blur(20px);
  border-top: 1px solid var(--bdr);
  z-index: 100;
}
.ctrl-btn {
  font-family: var(--mono);
  font-size: 11px;
  color: var(--dim);
  border: 1px solid var(--bdr);
  padding: 8px 14px;
  border-radius: var(--r);
  transition: color 0.15s, border-color 0.15s;
}
.ctrl-btn:hover { color: var(--br); border-color: var(--bdr2); }
.ctrl-btn.active { color: var(--teal); border-color: var(--teal-m); }
.ctrl-btn.end { color: var(--red); border-color: rgba(230,57,70,0.3); }
.ctrl-btn.leave { color: var(--dim); }
.mic-btn {
  font-family: var(--mono);
  font-size: 12px;
  width: 56px;
  height: 56px;
  border-radius: 50%;
  background: var(--dim2);
  color: var(--dim);
  border: 2px solid var(--bdr);
  transition: all 0.2s;
}
.mic-btn.on {
  background: var(--teal-d);
  color: var(--teal);
  border-color: var(--teal);
}

/* Modal */
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
.modal-close { font-size: 20px; color: var(--dim); }
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
.icon-picker { display: flex; gap: 6px; justify-content: center; }
.icon-opt {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  border: 1px solid var(--bdr);
  border-radius: var(--r);
  color: var(--dim);
  transition: all 0.15s;
}
.icon-opt.selected { color: var(--teal); border-color: var(--teal); background: var(--teal-d); }
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
}
.btn-primary:hover { opacity: 0.9; }

@media (max-width: 480px) {
  .controls-bar { padding-bottom: calc(12px + env(safe-area-inset-bottom, 0px)); }
  .ctrl-btn { padding: 6px 10px; font-size: 10px; }
  .mic-btn { width: 48px; height: 48px; font-size: 10px; }
  .participant-grid { gap: 8px; }
  .participant { width: 60px; }
  .p-avatar { width: 40px; height: 40px; font-size: 14px; }
}
</style>
