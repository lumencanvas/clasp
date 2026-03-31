<script setup>
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { useRelay } from '../composables/useRelay.js'
import { useToast } from '../composables/useToast.js'
import { useAudioMesh } from '../composables/useAudioMesh.js'
import RoomCard from '../components/spaces/RoomCard.vue'
import CreateRoomModal from '../components/spaces/CreateRoomModal.vue'
import ParticipantTile from '../components/spaces/ParticipantTile.vue'
import ChatPanel from '../components/spaces/ChatPanel.vue'
import ControlsBar from '../components/spaces/ControlsBar.vue'
import ToastContainer from '../components/ToastContainer.vue'

const { client, userName, authToken, connect, loginAsGuest } = useRelay()
const { toast } = useToast()

// --- Identity ---
const NAMES = ['Signal Ghost','Neon Drift','Pulse Echo','Wire Fox','Flux Moth','Byte Raven','Arc Shade','Phase Wolf','Glitch Owl','Node Lynx']
const COLORS = ['#00E5C8','#FF6B6B','#A78BFA','#F59E0B','#3B82F6','#EC4899','#10B981','#F97316','#06B6D4','#8B5CF6']
function rnd(a) { return a[Math.floor(Math.random() * a.length)] }
function aCol(id) { let h = 0; for (let i = 0; i < id.length; i++) h = (h << 5) - h + id.charCodeAt(i); return COLORS[Math.abs(h) % COLORS.length] }

const myId = ref(localStorage.getItem('clasp_spaces_id') || ('u_' + Date.now().toString(36) + Math.random().toString(36).slice(2, 6)))
const myName = ref('')
const myColor = computed(() => aCol(myId.value))
localStorage.setItem('clasp_spaces_id', myId.value)

const PREFIX = '/audiospace'
const DIR = `${PREFIX}/directory`

// --- State ---
const view = ref('lobby')
const connDot = ref(false)
const roomDirectory = reactive({})
const currentRoom = ref(null)
const myRole = ref('listener')
const handUp = ref(false)
const chatOpen = ref(false)
const chatMessages = ref([])
const participants = ref({})
const handState = ref({})
const searchQuery = ref('')
const showCreate = ref(false)
const showEnded = ref(false)

const unsubs = []
let dirUnsub = null
let presenceInterval = null

// --- Audio mesh ---
const audio = useAudioMesh(
  () => client.value,
  () => PREFIX,
  () => myId.value,
  () => currentRoom.value,
)

// --- Computed ---
const filteredRooms = computed(() => {
  const q = searchQuery.value.toLowerCase()
  return Object.values(roomDirectory)
    .filter(r => !r.__ended)
    .filter(r => !q || r.name?.toLowerCase().includes(q) || r.desc?.toLowerCase().includes(q) || r.hostName?.toLowerCase().includes(q))
    .sort((a, b) => (b.count || 0) - (a.count || 0) || b.createdAt - a.createdAt)
})

const speakers = computed(() => Object.values(participants.value).filter(p => !p.__left && (p.role === 'speaker' || p.isHost)))
const listeners = computed(() => {
  const list = Object.values(participants.value).filter(p => !p.__left && p.role === 'listener')
  return list.sort((a, b) => (handState.value[b.id] ? 1 : 0) - (handState.value[a.id] ? 1 : 0))
})
const handsCount = computed(() => Object.values(handState.value).filter(h => h?.raised).length)
const canSpeak = computed(() => myRole.value === 'host' || myRole.value === 'speaker')

// --- Room management ---
function createRoom({ name, desc, icon }) {
  const c = client.value
  if (!c) return
  const id = 'r_' + Date.now().toString(36) + Math.random().toString(36).slice(2, 6)
  const meta = { id, name, desc, icon, hostId: myId.value, hostName: myName.value, createdAt: Date.now(), count: 1 }
  c.set(`${DIR}/${id}`, meta)
  showCreate.value = false
  joinRoom(id, meta)
}

function joinRoom(roomId, metaOverride) {
  const c = client.value
  if (!c) { toast('Not connected to relay'); return }
  const meta = metaOverride || roomDirectory[roomId]
  if (!meta) return

  currentRoom.value = meta
  myRole.value = meta.hostId === myId.value ? 'host' : 'listener'
  handUp.value = false; chatOpen.value = false; chatMessages.value = []
  participants.value = {}; handState.value = {}
  cleanupRoom()
  view.value = 'room'

  const base = `${PREFIX}/rooms/${roomId}`

  unsubs.push(c.on(`${base}/participants/**`, (v, a) => {
    const uid = a.split('/').pop()
    if (!v || v.__left) {
      const p = { ...participants.value }; delete p[uid]; participants.value = p
      audio.destroyPeer(uid)
    } else {
      const isNew = !participants.value[uid]
      participants.value = { ...participants.value, [uid]: v }
      if (isNew && uid !== myId.value && uid > myId.value) audio.createPC(uid, true)
    }
    updateDirCount()
  }))

  unsubs.push(c.on(`${base}/speaking/**`, (v, a) => {
    const uid = a.split('/').pop()
    if (v) audio.speakingState.value = { ...audio.speakingState.value, [uid]: v }
  }))

  unsubs.push(c.on(`${base}/hands/**`, (v, a) => {
    const uid = a.split('/').pop()
    handState.value = { ...handState.value, [uid]: v }
  }))

  unsubs.push(c.on(`${base}/chat/**`, (v) => {
    if (v?.text) chatMessages.value = [...chatMessages.value, v].slice(-80)
  }))

  unsubs.push(c.on(`${base}/signal/${myId.value}/**`, (v, a) => {
    if (v) audio.handleSignal(a.split('/').pop(), v)
  }))

  unsubs.push(c.on(`${base}/roles/${myId.value}`, (v) => {
    if (!v || myRole.value === 'host') return
    if (v.role === 'speaker' && myRole.value !== 'speaker') {
      myRole.value = 'speaker'; toast('You were invited to speak!'); announcePresence()
    } else if (v.role === 'listener' && myRole.value === 'speaker') {
      myRole.value = 'listener'; audio.stopMic(); toast('Moved to listeners'); announcePresence()
    }
  }))

  unsubs.push(c.on(`${base}/ended`, (v) => {
    if (v?.ended && myRole.value !== 'host') showRoomEnded()
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
    color: myColor.value, isHost: myRole.value === 'host', ts: Date.now(),
  })
}

function updateDirCount() {
  const c = client.value
  if (!c || !currentRoom.value) return
  const count = Object.values(participants.value).filter(p => !p.__left).length
  c.set(`${DIR}/${currentRoom.value.id}`, { ...roomDirectory[currentRoom.value.id], ...currentRoom.value, count })
}

function leaveRoom() {
  const c = client.value
  if (c && currentRoom.value) {
    c.set(`${PREFIX}/rooms/${currentRoom.value.id}/participants/${myId.value}`, { __left: true, ts: Date.now() })
    c.set(`${PREFIX}/rooms/${currentRoom.value.id}/speaking/${myId.value}`, null)
    c.set(`${PREFIX}/rooms/${currentRoom.value.id}/hands/${myId.value}`, null)
  }
  cleanupRoom(); audio.cleanup()
  currentRoom.value = null; myRole.value = 'listener'
  view.value = 'lobby'
}

function endRoom() {
  if (!currentRoom.value || myRole.value !== 'host') return
  const c = client.value
  if (c) {
    c.set(`${PREFIX}/rooms/${currentRoom.value.id}/ended`, { ended: true, ts: Date.now() })
    c.set(`${DIR}/${currentRoom.value.id}`, { __ended: true, ts: Date.now() })
  }
  leaveRoom(); toast('Room ended')
}

function showRoomEnded() { showEnded.value = true; cleanupRoom(); audio.cleanup() }

function backFromEnded() {
  showEnded.value = false; currentRoom.value = null; myRole.value = 'listener'; view.value = 'lobby'
}

function cleanupRoom() {
  unsubs.forEach(u => { try { u() } catch {} }); unsubs.length = 0
  if (presenceInterval) { clearInterval(presenceInterval); presenceInterval = null }
}

// --- Host controls ---
function promote(uid) {
  const c = client.value
  if (myRole.value !== 'host' || !c || !currentRoom.value) return
  c.set(`${PREFIX}/rooms/${currentRoom.value.id}/roles/${uid}`, { role: 'speaker', ts: Date.now() })
  c.set(`${PREFIX}/rooms/${currentRoom.value.id}/hands/${uid}`, null)
  toast(`Invited ${participants.value[uid]?.name || uid} to speak`)
}

function demote(uid) {
  const c = client.value
  if (myRole.value !== 'host' || !c || !currentRoom.value || uid === myId.value) return
  c.set(`${PREFIX}/rooms/${currentRoom.value.id}/roles/${uid}`, { role: 'listener', ts: Date.now() })
  toast(`Moved ${participants.value[uid]?.name || uid} to listeners`)
}

// --- Mic ---
async function toggleMic() {
  if (!canSpeak.value) return
  if (audio.isMuted.value) {
    const ok = await audio.startMic()
    if (ok) { announcePresence(); toast('Microphone on') }
    else toast('Microphone access denied', 'err')
  } else {
    audio.stopMic(); announcePresence()
  }
}

function toggleHand() {
  const c = client.value
  if (!c || !currentRoom.value) return
  handUp.value = !handUp.value
  c.set(`${PREFIX}/rooms/${currentRoom.value.id}/hands/${myId.value}`, handUp.value ? { raised: true, ts: Date.now() } : null)
}

function sendChat(text) {
  const c = client.value
  if (!c || !currentRoom.value) return
  const id = Date.now().toString(36) + Math.random().toString(36).slice(2, 6)
  const msg = { text, name: myName.value, id: myId.value, color: myColor.value, ts: Date.now() }
  c.emit(`${PREFIX}/rooms/${currentRoom.value.id}/chat/${id}`, msg)
  chatMessages.value = [...chatMessages.value, msg].slice(-80)
}

// --- Lifecycle ---
onMounted(async () => {
  myName.value = userName.value || rnd(NAMES)

  try {
    if (!authToken.value) await loginAsGuest(myName.value)
    await connect()
    connDot.value = true

    const c = client.value
    if (!c) return

    dirUnsub = c.on(`${DIR}/**`, (val, addr) => {
      const id = addr.split('/').pop()
      if (!val || val.__ended) {
        if (currentRoom.value?.id === id && myRole.value !== 'host') showRoomEnded()
        delete roomDirectory[id]
      } else {
        roomDirectory[id] = val
      }
    })

    c.onConnect(() => { connDot.value = true; if (currentRoom.value) announcePresence() })
    c.onDisconnect(() => { connDot.value = false })
  } catch (e) {
    console.warn('[Spaces]', e)
    toast('Relay connection failed', 'err')
  }
})

onUnmounted(() => {
  if (currentRoom.value) leaveRoom()
  if (typeof dirUnsub === 'function') dirUnsub()
  audio.cleanup()
})
</script>

<template>
  <div class="spaces">
    <!-- STATUS BAR -->
    <div class="status-bar">
      <div class="logo-area">
        <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
          <circle cx="10" cy="10" r="8" stroke="var(--accent)" stroke-width="1.5"/>
          <circle cx="10" cy="10" r="3" fill="var(--accent)"/>
          <line x1="10" y1="2" x2="10" y2="5" stroke="var(--accent)" stroke-width="1.5"/>
          <line x1="10" y1="15" x2="10" y2="18" stroke="var(--accent)" stroke-width="1.5"/>
          <line x1="2" y1="10" x2="5" y2="10" stroke="var(--accent)" stroke-width="1.5"/>
          <line x1="15" y1="10" x2="18" y2="10" stroke="var(--accent)" stroke-width="1.5"/>
        </svg>
        <div>
          <div class="logo-text">AudioSpace</div>
          <div class="logo-sub">on CLASP</div>
        </div>
      </div>
      <div class="conn-status">
        <div class="conn-dot" :class="{ on: connDot }"></div>
        <span>{{ connDot ? 'demo-relay.clasp.to' : 'reconnecting...' }}</span>
      </div>
    </div>

    <!-- LOBBY -->
    <div v-if="view === 'lobby'" class="lobby">
      <div class="user-card">
        <div class="avatar" :style="{ width: '44px', height: '44px', fontSize: '16px', background: `linear-gradient(135deg, ${myColor}33, ${myColor}11)`, border: `2px solid ${myColor}88`, color: myColor }">
          {{ myId.slice(2, 4).toUpperCase() }}
        </div>
        <div class="user-info">
          <div class="user-name">{{ myName }}</div>
          <div class="user-id">{{ myId }}</div>
        </div>
      </div>

      <button class="btn btn-primary" @click="showCreate = true">+ New Room</button>

      <div class="search-row">
        <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--text3)" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
        <input v-model="searchQuery" class="input search-input" placeholder="search rooms..." />
      </div>

      <div class="section-label">LIVE ROOMS</div>

      <RoomCard v-for="room in filteredRooms" :key="room.id" :room="room" @join="joinRoom" />

      <div v-if="!filteredRooms.length" class="empty-state">
        No rooms yet.<br>Create one to get started.
      </div>
    </div>

    <!-- ROOM VIEW -->
    <div v-if="view === 'room' && currentRoom" class="room-view">
      <div class="room-header">
        <button class="back-btn" @click="leaveRoom">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
        </button>
        <div class="room-header-info">
          <span class="room-header-icon">{{ currentRoom.icon }}</span>
          <span class="room-header-name">{{ currentRoom.name }}</span>
          <span v-if="myRole === 'host'" class="host-badge">HOST</span>
        </div>
        <div class="live-tag"><div class="live-dot"></div>LIVE</div>
      </div>

      <!-- STAGE -->
      <div class="stage">
        <div class="section-label">ON STAGE</div>
        <div class="speakers-grid">
          <ParticipantTile
            v-for="p in speakers" :key="p.id"
            :participant="p"
            :size="60"
            :speaking="audio.speakingState.value[p.id]"
            :is-host="myRole === 'host'"
            mode="speaker"
            @demote="demote"
          />
          <div v-if="!speakers.length" class="empty-stage">stage is empty</div>
        </div>
      </div>

      <!-- LISTENERS -->
      <div class="listeners-section">
        <div class="section-label">
          LISTENERS ({{ listeners.length }})
          <span v-if="handsCount" class="hands-count">{{ handsCount }} hands</span>
        </div>
        <div class="listeners-grid">
          <ParticipantTile
            v-for="p in listeners" :key="p.id"
            :participant="p"
            :size="42"
            :speaking="audio.speakingState.value[p.id]"
            :is-host="myRole === 'host'"
            :hand-raised="!!handState[p.id]?.raised"
            mode="listener"
            @promote="promote"
          />
        </div>
      </div>

      <ChatPanel :messages="chatMessages" :show="chatOpen" @send="sendChat" />

      <ControlsBar
        :role="myRole"
        :is-muted="audio.isMuted.value"
        :chat-open="chatOpen"
        :hand-up="handUp"
        :my-volume="audio.myVolume.value"
        @toggle-chat="chatOpen = !chatOpen"
        @toggle-hand="toggleHand"
        @toggle-mic="toggleMic"
        @end-room="endRoom"
        @leave="leaveRoom"
      />
    </div>

    <!-- ROOM ENDED -->
    <div v-if="showEnded" class="modal-overlay center">
      <div class="modal centered">
        <div style="font-size: 36px; text-align: center; opacity: 0.3;">&times;</div>
        <h2 style="text-align: center; font-size: 18px; color: var(--text);">Room Ended</h2>
        <p style="text-align: center; color: var(--text2); font-size: 14px;">The host closed this room.</p>
        <button class="btn btn-primary" style="width: 100%; margin-top: 12px;" @click="backFromEnded">Back to Lobby</button>
      </div>
    </div>

    <CreateRoomModal :show="showCreate" @close="showCreate = false" @create="createRoom" />
    <ToastContainer />
  </div>
</template>

<style src="./spaces-styles.css" scoped></style>
