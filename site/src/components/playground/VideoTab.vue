<script setup>
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { useClasp } from '../../composables/useClasp'
import { useVideoCall } from '../../composables/useVideoCall'
import { useVideoStream, QUALITY_PRESETS } from '../../composables/useVideoStream'

const { connected, sessionId, settings } = useClasp()

// Mode: 'relay' is the primary mode (CLASP native), 'p2p' is optional WebRTC
const mode = ref('relay')

// P2P mode
const videoCall = useVideoCall()

// Relay mode
const videoStream = useVideoStream()

// Common state
const room = ref('lobby')
const nickname = ref('')
const inRoom = ref(false)
const localVideoRef = ref(null)
const remoteCanvasRefs = ref(new Map()) // peerId -> canvas element
const remoteCanvasContexts = ref(new Map()) // peerId -> cached 2d context
let canvasRenderLoop = null
let webCodecsSupported = true

// Quality settings (visible upfront)
const qualityPreset = ref('low')
const customBitrate = ref(400)
const customFramerate = ref(24)

// Popular rooms
const popularRooms = ['lobby', 'demo', 'test', 'dev']

// Computed: participants list
const participants = computed(() => {
  if (mode.value === 'p2p') {
    return videoCall.participantList.value
  } else {
    return videoStream.participantList.value
  }
})

// Computed: peer streams (P2P mode)
const peerStreams = computed(() => {
  if (mode.value === 'p2p') {
    return videoCall.peerList.value.filter(p => p.stream)
  }
  return []
})

// Computed: broadcaster streams (Relay mode)
const broadcasters = computed(() => {
  if (mode.value === 'relay') {
    return videoStream.broadcasterList.value
  }
  return []
})

// Computed: error message
const error = computed(() => {
  return mode.value === 'p2p' ? videoCall.error.value : videoStream.error.value
})

// Computed: is broadcasting (relay mode only)
const isBroadcasting = computed(() => {
  return mode.value === 'relay' && videoStream.isBroadcasting.value
})

// Computed: has local stream
const hasLocalStream = computed(() => {
  return mode.value === 'p2p'
    ? !!videoCall.localStream.value
    : !!videoStream.localStream.value
})

// Callback ref for local video element - sets srcObject when DOM element mounts
function setLocalVideoRef(el) {
  localVideoRef.value = el
  if (el) {
    const stream = mode.value === 'p2p' ? videoCall.localStream.value : videoStream.localStream.value
    if (stream) {
      el.srcObject = stream
      el.play().catch(() => {})
    }
  }
}

// Watch local stream changes AND inRoom state to reapply srcObject after room join
watch(
  [() => mode.value === 'p2p' ? videoCall.localStream.value : videoStream.localStream.value, inRoom],
  ([stream]) => {
    nextTick(() => {
      if (localVideoRef.value && stream) {
        localVideoRef.value.srcObject = stream
        localVideoRef.value.play().catch(() => {})
      }
    })
  }
)

// Watch quality preset changes
watch(qualityPreset, (preset) => {
  if (preset !== 'custom') {
    videoStream.setQuality(preset)
  }
})

// Start/stop canvas render loop for relay mode
function startCanvasRenderLoop() {
  if (canvasRenderLoop) return

  const render = () => {
    if (!inRoom.value || mode.value !== 'relay') {
      canvasRenderLoop = null
      return
    }

    // Copy content from source canvases to display canvases
    for (const [peerId, displayCanvas] of remoteCanvasRefs.value) {
      const sourceCanvas = videoStream.getRemoteCanvas(peerId)
      if (sourceCanvas && displayCanvas) {
        let ctx = remoteCanvasContexts.value.get(peerId)
        if (!ctx) {
          ctx = displayCanvas.getContext('2d')
          remoteCanvasContexts.value.set(peerId, ctx)
        }

        if (displayCanvas.width !== sourceCanvas.width || displayCanvas.height !== sourceCanvas.height) {
          displayCanvas.width = sourceCanvas.width || 640
          displayCanvas.height = sourceCanvas.height || 480
        }
        ctx.drawImage(sourceCanvas, 0, 0)
      }
    }

    canvasRenderLoop = requestAnimationFrame(render)
  }

  canvasRenderLoop = requestAnimationFrame(render)
}

function stopCanvasRenderLoop() {
  if (canvasRenderLoop) {
    cancelAnimationFrame(canvasRenderLoop)
    canvasRenderLoop = null
  }
  remoteCanvasContexts.value.clear()
}

// Set up canvas ref for a broadcaster
function setRemoteCanvasRef(peerId, el) {
  if (el) {
    remoteCanvasRefs.value.set(peerId, el)
  } else {
    remoteCanvasRefs.value.delete(peerId)
    remoteCanvasContexts.value.delete(peerId)
  }
}

// Start camera preview
async function startCamera() {
  try {
    if (mode.value === 'p2p') {
      await videoCall.getUserMedia()
    } else {
      await videoStream.getUserMedia()
    }
    nextTick(() => {
      if (localVideoRef.value) {
        localVideoRef.value.srcObject = mode.value === 'p2p'
          ? videoCall.localStream.value
          : videoStream.localStream.value
      }
    })
  } catch (e) {
    console.error('Failed to start camera:', e)
  }
}

// Stop camera
function stopCamera() {
  if (mode.value === 'p2p') {
    videoCall.stopUserMedia()
  } else {
    videoStream.stopUserMedia()
  }
}

// Join room - can join as viewer or broadcaster
async function joinRoom(roomName = null, startBroadcast = false) {
  if (!connected.value || !nickname.value.trim()) return

  if (roomName) {
    room.value = roomName
  }

  // Apply quality settings before joining (for relay mode)
  if (mode.value === 'relay') {
    if (qualityPreset.value !== 'custom') {
      videoStream.setQuality(qualityPreset.value)
    } else {
      videoStream.setCustomQuality({
        bitrate: customBitrate.value * 1000,
        framerate: customFramerate.value,
      })
    }
  }

  if (mode.value === 'p2p') {
    // P2P mode requires camera
    const localStream = videoCall.localStream.value
    if (!localStream) {
      await startCamera()
    }
    await videoCall.joinRoom(room.value, nickname.value)
  } else {
    await videoStream.joinRoom(room.value, nickname.value)
    // Start canvas render loop for relay mode
    startCanvasRenderLoop()

    // Only start broadcasting if requested and camera is ready
    if (startBroadcast) {
      if (!videoStream.localStream.value) {
        await startCamera()
      }
      await videoStream.startBroadcasting()
    }
  }

  inRoom.value = true
}

// Join as viewer (no camera needed)
async function joinAsViewer(roomName = null) {
  await joinRoom(roomName, false)
}

// Join and start broadcasting
async function joinAndBroadcast(roomName = null) {
  await joinRoom(roomName, true)
}

// Leave room
function leaveRoom() {
  stopCanvasRenderLoop()
  remoteCanvasRefs.value.clear()
  remoteCanvasContexts.value.clear()

  if (mode.value === 'p2p') {
    videoCall.leaveRoom()
  } else {
    videoStream.leaveRoom()
  }
  inRoom.value = false
}

// Toggle audio (P2P mode)
function toggleAudio() {
  if (mode.value === 'p2p') {
    videoCall.toggleAudio()
  }
}

// Toggle video (P2P mode)
function toggleVideo() {
  if (mode.value === 'p2p') {
    videoCall.toggleVideo()
  }
}

// Start broadcasting (Relay mode)
async function startBroadcasting() {
  if (mode.value === 'relay') {
    // Start camera if not already running
    if (!videoStream.localStream.value) {
      await startCamera()
    }
    await videoStream.startBroadcasting()

    // Update local video element
    nextTick(() => {
      if (localVideoRef.value && videoStream.localStream.value) {
        localVideoRef.value.srcObject = videoStream.localStream.value
      }
    })
  }
}

// Stop broadcasting (Relay mode)
function stopBroadcasting() {
  if (mode.value === 'relay') {
    videoStream.stopBroadcasting()
  }
}

// Apply custom quality settings
function applyCustomQuality() {
  qualityPreset.value = 'custom'
  videoStream.setCustomQuality({
    bitrate: customBitrate.value * 1000,
    framerate: customFramerate.value,
  })
}

// Share screen (P2P mode)
async function shareScreen() {
  if (mode.value === 'p2p') {
    await videoCall.shareScreen()
  }
}

// Switch mode
function switchMode(newMode) {
  if (inRoom.value) {
    leaveRoom()
  }
  stopCamera()
  mode.value = newMode

  // Check WebCodecs support when switching to relay mode
  if (newMode === 'relay') {
    webCodecsSupported = videoStream.checkWebCodecsSupport()
  }
}

// Copy room link to clipboard
function copyRoomLink() {
  const url = new URL(window.location.href)
  url.searchParams.set('room', room.value)
  url.searchParams.set('mode', mode.value)
  navigator.clipboard.writeText(url.toString()).catch(() => {})
}

// Connection status
const connectionStatus = computed(() => {
  if (!connected.value) return 'disconnected'
  if (error.value) return 'error'
  return 'connected'
})

// Get avatar color
function getAvatarColor(name) {
  if (!name) return '#607D8B'
  const colors = ['#FF5F1F', '#2196F3', '#4CAF50', '#9C27B0', '#FF9800', '#00BCD4', '#E91E63', '#607D8B']
  let hash = 0
  for (let i = 0; i < name.length; i++) {
    hash = name.charCodeAt(i) + ((hash << 5) - hash)
  }
  return colors[Math.abs(hash) % colors.length]
}

// Get initials
function getInitials(name) {
  if (!name) return '?'
  const parts = name.trim().split(' ').filter(Boolean)
  if (parts.length === 0) return '?'
  return parts.map(n => n[0]).join('').toUpperCase().slice(0, 2)
}

// Format bitrate
function formatBitrate(bps) {
  if (bps >= 1_000_000) {
    return `${(bps / 1_000_000).toFixed(1)} Mbps`
  }
  return `${Math.round(bps / 1000)} Kbps`
}

// Set default nickname and check WebCodecs
onMounted(() => {
  nickname.value = settings.name || 'Anonymous'
  webCodecsSupported = videoStream.checkWebCodecsSupport()
})

// Cleanup
onUnmounted(() => {
  stopCanvasRenderLoop()
  leaveRoom()
  stopCamera()
})

// Watch connection state
watch(connected, (isConnected) => {
  if (!isConnected && inRoom.value) {
    leaveRoom()
  }
})
</script>

<template>
  <div class="video-tab">
    <!-- Join Screen -->
    <div v-if="!inRoom" class="join-screen">
      <div class="join-card">
        <div class="join-header">
          <svg class="video-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <polygon points="23 7 16 12 23 17 23 7"/>
            <rect x="1" y="5" width="15" height="14" rx="2" ry="2"/>
          </svg>
          <h2>CLASP Video</h2>
          <p>Real-time video communication powered by CLASP</p>
        </div>

        <!-- Mode Toggle -->
        <div class="mode-toggle">
          <button
            :class="['mode-btn', { active: mode === 'p2p' }]"
            @click="switchMode('p2p')"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
              <circle cx="9" cy="7" r="4"/>
              <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
              <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
            </svg>
            P2P WebRTC
          </button>
          <button
            :class="['mode-btn', { active: mode === 'relay' }]"
            @click="switchMode('relay')"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <rect x="2" y="2" width="20" height="8" rx="2" ry="2"/>
              <rect x="2" y="14" width="20" height="8" rx="2" ry="2"/>
              <line x1="6" y1="6" x2="6.01" y2="6"/>
              <line x1="6" y1="18" x2="6.01" y2="18"/>
            </svg>
            CLASP Relay
          </button>
        </div>

        <div class="mode-description">
          <template v-if="mode === 'p2p'">
            <strong>P2P Mode:</strong> Direct WebRTC connections between peers. Best quality, lowest latency.
            CLASP handles signaling only.
          </template>
          <template v-else>
            <strong>Relay Mode:</strong> Video encoded and streamed through CLASP router.
            Works with any client/transport. Uses WebCodecs for H.264 encoding.
            <p v-if="!webCodecsSupported" class="webcodecs-warning">
              WebCodecs is not supported in this browser. Relay mode may not work.
            </p>
          </template>
        </div>

        <!-- Quality Settings (Relay mode - shown upfront) -->
        <div v-if="mode === 'relay'" class="quality-section">
          <div class="section-label">Encoding Quality</div>
          <div class="preset-buttons">
            <button
              v-for="(preset, name) in QUALITY_PRESETS"
              :key="name"
              :class="['preset-btn', { active: qualityPreset === name }]"
              @click="qualityPreset = name"
            >
              <span class="preset-name">{{ name }}</span>
              <span class="preset-detail">{{ preset.height }}p @ {{ Math.round(preset.bitrate / 1000) }}k</span>
            </button>
          </div>
          <div class="quality-sliders">
            <div class="slider-group">
              <label>Bitrate: {{ customBitrate }} Kbps</label>
              <input
                type="range"
                v-model="customBitrate"
                min="200"
                max="3000"
                step="100"
                @input="qualityPreset = 'custom'"
              />
            </div>
            <div class="slider-group">
              <label>Framerate: {{ customFramerate }} fps</label>
              <input
                type="range"
                v-model="customFramerate"
                min="15"
                max="60"
                step="5"
                @input="qualityPreset = 'custom'"
              />
            </div>
          </div>
        </div>

        <!-- Camera Preview -->
        <div class="preview-section">
          <div class="preview-container">
            <video
              :ref="setLocalVideoRef"
              autoplay
              muted
              playsinline
              class="preview-video"
              @loadedmetadata="$event.target.play()"
            />
            <div v-if="!hasLocalStream" class="preview-placeholder">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
                <polygon points="23 7 16 12 23 17 23 7"/>
                <rect x="1" y="5" width="15" height="14" rx="2" ry="2"/>
              </svg>
              <span>Camera preview</span>
            </div>
          </div>
          <button class="preview-btn" @click="startCamera" :disabled="!connected">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polygon points="23 7 16 12 23 17 23 7"/>
              <rect x="1" y="5" width="15" height="14" rx="2" ry="2"/>
            </svg>
            Start Camera
          </button>
        </div>

        <div class="join-form">
          <div class="field">
            <label>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                <circle cx="12" cy="7" r="4"/>
              </svg>
              Nickname
            </label>
            <input
              v-model="nickname"
              type="text"
              placeholder="Enter your name"
              :disabled="!connected"
              @keyup.enter="joinRoom()"
            />
          </div>

          <div class="field">
            <label>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <rect x="3" y="3" width="7" height="7"/>
                <rect x="14" y="3" width="7" height="7"/>
                <rect x="14" y="14" width="7" height="7"/>
                <rect x="3" y="14" width="7" height="7"/>
              </svg>
              Room
            </label>
            <div class="room-input-wrapper">
              <span class="room-prefix">#</span>
              <input
                v-model="room"
                type="text"
                placeholder="lobby"
                :disabled="!connected"
                @keyup.enter="joinRoom()"
              />
            </div>
          </div>

          <div class="popular-rooms">
            <span class="rooms-label">Quick join:</span>
            <button
              v-for="r in popularRooms"
              :key="r"
              :class="['room-chip', { active: room === r }]"
              @click="room = r"
              :disabled="!connected"
            >
              #{{ r }}
            </button>
          </div>

          <div v-if="mode === 'relay'" class="join-buttons">
            <button
              class="join-btn secondary"
              @click="joinAsViewer()"
              :disabled="!connected || !nickname.trim()"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                <circle cx="12" cy="12" r="3"/>
              </svg>
              Watch Only
            </button>
            <button
              class="join-btn primary"
              @click="joinAndBroadcast()"
              :disabled="!connected || !nickname.trim()"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="2"/>
                <path d="M16.24 7.76a6 6 0 0 1 0 8.49m-8.48-.01a6 6 0 0 1 0-8.49"/>
              </svg>
              Join & Broadcast
            </button>
          </div>
          <button
            v-else
            class="join-btn primary"
            @click="joinRoom()"
            :disabled="!connected || !nickname.trim()"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polygon points="23 7 16 12 23 17 23 7"/>
              <rect x="1" y="5" width="15" height="14" rx="2" ry="2"/>
            </svg>
            Join Room
          </button>

          <p v-if="error" class="error-message">{{ error }}</p>

          <p v-if="!connected" class="connect-hint">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <circle cx="12" cy="12" r="10"/>
              <line x1="12" y1="8" x2="12" y2="12"/>
              <line x1="12" y1="16" x2="12.01" y2="16"/>
            </svg>
            Connect to a CLASP server first
          </p>
        </div>
      </div>
    </div>

    <!-- Video Room -->
    <div v-else class="video-room">
      <!-- Room Header -->
      <div class="room-header">
        <div class="room-info">
          <button class="back-btn" @click="leaveRoom">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="15 18 9 12 15 6"/>
            </svg>
          </button>
          <div class="room-details">
            <h3>#{{ room }}</h3>
            <span class="mode-badge">{{ mode === 'p2p' ? 'P2P' : 'Relay' }}</span>
            <span :class="['connection-dot', connectionStatus]"></span>
            <span class="member-count">{{ participants.length + 1 }} in room</span>
          </div>
        </div>
        <div class="room-actions">
          <button class="action-btn" @click="copyRoomLink" title="Copy Room Link">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
            </svg>
          </button>
          <button
            v-if="mode === 'relay'"
            :class="['broadcast-toggle', { active: isBroadcasting }]"
            @click="isBroadcasting ? stopBroadcasting() : startBroadcasting()"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <circle cx="12" cy="12" r="2"/>
              <path d="M16.24 7.76a6 6 0 0 1 0 8.49m-8.48-.01a6 6 0 0 1 0-8.49m11.31-2.82a10 10 0 0 1 0 14.14m-14.14 0a10 10 0 0 1 0-14.14"/>
            </svg>
            {{ isBroadcasting ? 'Stop Broadcast' : 'Start Broadcast' }}
          </button>
          <button
            v-if="mode === 'p2p'"
            :class="['action-btn', { muted: !videoCall.audioEnabled.value }]"
            @click="toggleAudio"
            title="Toggle Microphone"
          >
            <svg v-if="videoCall.audioEnabled.value" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"/>
              <path d="M19 10v2a7 7 0 0 1-14 0v-2"/>
              <line x1="12" y1="19" x2="12" y2="23"/>
              <line x1="8" y1="23" x2="16" y2="23"/>
            </svg>
            <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <line x1="1" y1="1" x2="23" y2="23"/>
              <path d="M9 9v3a3 3 0 0 0 5.12 2.12M15 9.34V4a3 3 0 0 0-5.94-.6"/>
              <path d="M17 16.95A7 7 0 0 1 5 12v-2m14 0v2a7 7 0 0 1-.11 1.23"/>
              <line x1="12" y1="19" x2="12" y2="23"/>
              <line x1="8" y1="23" x2="16" y2="23"/>
            </svg>
          </button>
          <button
            v-if="mode === 'p2p'"
            :class="['action-btn', { muted: !videoCall.videoEnabled.value }]"
            @click="toggleVideo"
            title="Toggle Camera"
          >
            <svg v-if="videoCall.videoEnabled.value" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <polygon points="23 7 16 12 23 17 23 7"/>
              <rect x="1" y="5" width="15" height="14" rx="2" ry="2"/>
            </svg>
            <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M16 16v1a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2h2m5.66 0H14a2 2 0 0 1 2 2v3.34l1 1L23 7v10"/>
              <line x1="1" y1="1" x2="23" y2="23"/>
            </svg>
          </button>
          <button
            v-if="mode === 'p2p'"
            class="action-btn"
            @click="shareScreen"
            title="Share Screen"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <rect x="2" y="3" width="20" height="14" rx="2" ry="2"/>
              <line x1="8" y1="21" x2="16" y2="21"/>
              <line x1="12" y1="17" x2="12" y2="21"/>
            </svg>
          </button>
        </div>
      </div>

      <!-- Live Stats Bar (Relay mode) -->
      <div v-if="mode === 'relay'" class="stats-bar">
        <div class="stat status">
          <span class="stat-label">Status</span>
          <span :class="['stat-value', isBroadcasting ? 'live' : 'viewer']">
            {{ isBroadcasting ? 'Broadcasting' : 'Viewing' }}
          </span>
        </div>
        <template v-if="isBroadcasting">
          <div class="stat">
            <span class="stat-label">Encoding</span>
            <span class="stat-value">{{ videoStream.quality.height }}p @ {{ videoStream.quality.framerate }}fps</span>
          </div>
          <div class="stat">
            <span class="stat-label">Bitrate</span>
            <span class="stat-value">{{ formatBitrate(videoStream.quality.bitrate) }}</span>
          </div>
          <div class="stat">
            <span class="stat-label">Sent</span>
            <span class="stat-value">{{ videoStream.stats.framesSent }}</span>
          </div>
        </template>
        <div class="stat">
          <span class="stat-label">Received</span>
          <span class="stat-value">{{ videoStream.stats.framesReceived }}</span>
        </div>
        <div class="stat">
          <span class="stat-label">Bandwidth</span>
          <span class="stat-value">{{ formatBitrate(videoStream.stats.bytesPerSecond * 8) }}</span>
        </div>
        <div class="stat">
          <span class="stat-label">Streams</span>
          <span class="stat-value">{{ broadcasters.length }}</span>
        </div>
      </div>

      <!-- Video Grid -->
      <div class="video-layout">
        <div class="video-grid" :class="{ 'single': peerStreams.length === 0 && broadcasters.length === 0 && !isBroadcasting }">
          <!-- Local video (only show if broadcasting or in P2P mode) -->
          <div v-if="mode === 'p2p' || isBroadcasting" class="video-cell local">
            <video
              :ref="setLocalVideoRef"
              autoplay
              muted
              playsinline
              class="video-element"
              @loadedmetadata="$event.target.play()"
            />
            <div class="video-label">
              <span class="name">{{ nickname }} (you)</span>
              <span v-if="mode === 'relay' && isBroadcasting" class="broadcast-indicator">LIVE</span>
            </div>
          </div>

          <!-- Viewer mode prompt (relay mode, not broadcasting) -->
          <div v-if="mode === 'relay' && !isBroadcasting && broadcasters.length === 0" class="viewer-prompt">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
              <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
              <circle cx="12" cy="12" r="3"/>
            </svg>
            <h4>Watching Room</h4>
            <p>Waiting for someone to broadcast...</p>
            <button class="start-broadcast-btn" @click="startBroadcasting">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="2"/>
                <path d="M16.24 7.76a6 6 0 0 1 0 8.49m-8.48-.01a6 6 0 0 1 0-8.49"/>
              </svg>
              Start Broadcasting
            </button>
          </div>

          <!-- P2P Mode: Remote peer videos -->
          <template v-if="mode === 'p2p'">
            <div
              v-for="peer in peerStreams"
              :key="peer.id"
              class="video-cell"
            >
              <video
                :ref="(el) => { if (el && peer.stream) el.srcObject = peer.stream }"
                autoplay
                muted
                playsinline
                class="video-element"
                @loadedmetadata="$event.target.play()"
              />
              <div class="video-label">
                <span class="name">{{ peer.name }}</span>
                <span v-if="!peer.audioEnabled" class="muted-indicator">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <line x1="1" y1="1" x2="23" y2="23"/>
                    <path d="M9 9v3a3 3 0 0 0 5.12 2.12"/>
                  </svg>
                </span>
              </div>
            </div>
          </template>

          <!-- Relay Mode: Broadcaster canvases -->
          <template v-if="mode === 'relay'">
            <div
              v-for="broadcaster in broadcasters"
              :key="broadcaster.id"
              class="video-cell"
            >
              <canvas
                :ref="(el) => setRemoteCanvasRef(broadcaster.id, el)"
                class="video-canvas"
              />
              <div class="video-label">
                <span class="name">{{ broadcaster.name }}</span>
                <span class="broadcast-indicator">LIVE</span>
              </div>
            </div>
          </template>
        </div>

        <!-- Participants Sidebar -->
        <div class="participants-sidebar">
          <div class="sidebar-header">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
              <circle cx="9" cy="7" r="4"/>
              <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
              <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
            </svg>
            Participants
          </div>
          <div class="participants-list">
            <!-- Self -->
            <div class="participant self">
              <div class="participant-avatar" :style="{ background: getAvatarColor(nickname) }">
                {{ getInitials(nickname) }}
              </div>
              <span class="participant-name">{{ nickname }}</span>
              <span class="you-tag">you</span>
              <span v-if="mode === 'relay' && isBroadcasting" class="live-dot"></span>
              <span v-else class="online-indicator"></span>
            </div>

            <!-- Other participants -->
            <div
              v-for="p in participants"
              :key="p.id"
              class="participant"
            >
              <div class="participant-avatar" :style="{ background: getAvatarColor(p.name) }">
                {{ getInitials(p.name) }}
              </div>
              <span class="participant-name">{{ p.name }}</span>
              <span v-if="p.isBroadcaster" class="live-dot"></span>
              <span v-else class="online-indicator"></span>
            </div>
          </div>
        </div>
      </div>

      <!-- Error display -->
      <div v-if="error" class="error-banner">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <circle cx="12" cy="12" r="10"/>
          <line x1="12" y1="8" x2="12" y2="12"/>
          <line x1="12" y1="16" x2="12.01" y2="16"/>
        </svg>
        {{ error }}
      </div>
    </div>
  </div>
</template>

<style scoped>
.video-tab {
  height: 100%;
  display: flex;
  flex-direction: column;
}

/* Join Screen */
.join-screen {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
}

.join-card {
  width: 100%;
  max-width: 500px;
  background: rgba(255,255,255,0.6);
  border: 1px solid rgba(0,0,0,0.1);
  padding: 1.5rem;
}

.join-header {
  text-align: center;
  margin-bottom: 1.5rem;
}

.join-header .video-icon {
  width: 48px;
  height: 48px;
  margin-bottom: 1rem;
  opacity: 0.3;
}

.join-header h2 {
  margin: 0 0 0.5rem;
  font-size: 1.4rem;
  letter-spacing: 0.15em;
  font-weight: 500;
}

.join-header p {
  margin: 0;
  font-size: 0.85rem;
  opacity: 0.6;
  line-height: 1.5;
}

/* Mode Toggle */
.mode-toggle {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.mode-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.8rem;
  background: transparent;
  border: 1px solid rgba(0,0,0,0.12);
  font-family: inherit;
  font-size: 0.8rem;
  letter-spacing: 0.05em;
  cursor: pointer;
  transition: all 0.15s;
  min-height: 44px;
}

.mode-btn:hover {
  background: rgba(0,0,0,0.03);
}

.mode-btn.active {
  background: var(--ink);
  color: var(--paper);
  border-color: var(--ink);
}

.mode-btn svg {
  width: 18px;
  height: 18px;
}

.mode-description {
  padding: 0.75rem 1rem;
  background: rgba(0,0,0,0.03);
  font-size: 0.8rem;
  line-height: 1.5;
  margin-bottom: 1.5rem;
}

.mode-description strong {
  color: var(--accent);
}

.webcodecs-warning {
  margin: 0.5rem 0 0;
  padding: 0.5rem;
  background: rgba(244, 67, 54, 0.1);
  color: #d32f2f;
  font-size: 0.75rem;
}

/* Quality Section (Join Screen) */
.quality-section {
  background: rgba(0,0,0,0.03);
  padding: 1rem;
  margin-bottom: 1rem;
  border: 1px solid rgba(0,0,0,0.08);
}

.section-label {
  font-size: 0.7rem;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  opacity: 0.5;
  margin-bottom: 0.75rem;
}

.preset-buttons {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.preset-btn {
  padding: 0.6rem 0.5rem;
  background: transparent;
  border: 1px solid rgba(0,0,0,0.12);
  font-family: inherit;
  cursor: pointer;
  transition: all 0.15s;
  text-align: center;
  min-height: 44px;
}

.preset-btn:hover {
  background: rgba(0,0,0,0.03);
}

.preset-btn.active {
  background: var(--ink);
  color: var(--paper);
  border-color: var(--ink);
}

.preset-name {
  display: block;
  font-size: 0.8rem;
  text-transform: capitalize;
  font-weight: 500;
}

.preset-detail {
  display: block;
  font-size: 0.65rem;
  opacity: 0.6;
  margin-top: 0.2rem;
}

.preset-btn.active .preset-detail {
  opacity: 0.8;
}

.quality-sliders {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.slider-group {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}

.slider-group label {
  font-size: 0.75rem;
  opacity: 0.7;
}

.slider-group input[type="range"] {
  width: 100%;
  accent-color: var(--accent);
}

/* Preview Section */
.preview-section {
  margin-bottom: 1.5rem;
}

.preview-container {
  position: relative;
  aspect-ratio: 4/3;
  background: rgba(0,0,0,0.05);
  border: 1px solid rgba(0,0,0,0.1);
  overflow: hidden;
  margin-bottom: 0.75rem;
}

.preview-video {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transform: scaleX(-1);
}

.preview-placeholder {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  color: rgba(0,0,0,0.3);
}

.preview-placeholder svg {
  width: 48px;
  height: 48px;
}

.preview-placeholder span {
  font-size: 0.85rem;
}

.preview-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.6rem;
  background: rgba(0,0,0,0.05);
  border: 1px solid rgba(0,0,0,0.1);
  font-family: inherit;
  font-size: 0.8rem;
  cursor: pointer;
  transition: all 0.15s;
}

.preview-btn:hover:not(:disabled) {
  background: rgba(0,0,0,0.08);
}

.preview-btn:disabled {
  opacity: 0.5;
}

.preview-btn svg {
  width: 16px;
  height: 16px;
}

/* Join Form */
.join-form {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.field label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.75rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  opacity: 0.6;
}

.field label svg {
  width: 14px;
  height: 14px;
}

.field input {
  padding: 0.8rem 1rem;
  border: 1px solid rgba(0,0,0,0.12);
  background: rgba(255,255,255,0.8);
  color: var(--ink, #1a1a1a);
  font-family: inherit;
  font-size: 1rem;
  transition: border-color 0.15s;
}

.field input:focus {
  outline: none;
  border-color: var(--accent);
}

.field input:disabled {
  opacity: 0.5;
}

.room-input-wrapper {
  display: flex;
  align-items: center;
  border: 1px solid rgba(0,0,0,0.12);
  background: rgba(255,255,255,0.8);
}

.room-input-wrapper:focus-within {
  border-color: var(--accent);
}

.room-prefix {
  padding: 0.8rem 0 0.8rem 1rem;
  opacity: 0.4;
  font-size: 0.95rem;
}

.room-input-wrapper input {
  border: none;
  padding-left: 0.25rem;
  flex: 1;
}

.popular-rooms {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.rooms-label {
  font-size: 0.7rem;
  opacity: 0.5;
  letter-spacing: 0.05em;
}

.room-chip {
  padding: 0.35rem 0.6rem;
  font-size: 0.75rem;
  border: 1px solid rgba(0,0,0,0.12);
  background: transparent;
  cursor: pointer;
  font-family: inherit;
  transition: all 0.15s;
}

.room-chip:hover:not(:disabled) {
  background: rgba(0,0,0,0.05);
}

.room-chip.active {
  background: var(--ink);
  color: var(--paper);
  border-color: var(--ink);
}

.room-chip:disabled {
  opacity: 0.4;
}

.join-buttons {
  display: grid;
  grid-template-columns: 1fr;
  gap: 0.75rem;
  margin-top: 0.5rem;
}

.join-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.9rem 1rem;
  border: none;
  font-family: inherit;
  font-size: 0.85rem;
  letter-spacing: 0.08em;
  cursor: pointer;
  transition: all 0.15s;
  min-height: 44px;
}

.join-btn.primary {
  background: var(--ink);
  color: var(--paper);
}

.join-btn.secondary {
  background: transparent;
  color: var(--ink);
  border: 1px solid rgba(0,0,0,0.2);
}

.join-btn svg {
  width: 18px;
  height: 18px;
}

.join-btn.primary:hover:not(:disabled) {
  background: var(--accent);
}

.join-btn.secondary:hover:not(:disabled) {
  background: rgba(0,0,0,0.05);
  border-color: rgba(0,0,0,0.3);
}

.join-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.error-message {
  margin: 0;
  padding: 0.75rem;
  background: rgba(244, 67, 54, 0.1);
  color: #d32f2f;
  font-size: 0.85rem;
  text-align: center;
}

.connect-hint {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  margin: 0;
  font-size: 0.8rem;
  opacity: 0.5;
  text-align: center;
}

.connect-hint svg {
  width: 16px;
  height: 16px;
}

/* Video Room */
.video-room {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: rgba(255,255,255,0.3);
  border: 1px solid rgba(0,0,0,0.1);
  min-height: 0;
}

.room-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  background: rgba(255,255,255,0.5);
  border-bottom: 1px solid rgba(0,0,0,0.08);
}

.room-info {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.back-btn {
  min-width: 44px;
  min-height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid rgba(0,0,0,0.1);
  cursor: pointer;
  transition: all 0.15s;
}

.back-btn:hover {
  background: rgba(0,0,0,0.05);
  border-color: rgba(0,0,0,0.2);
}

.back-btn svg {
  width: 16px;
  height: 16px;
}

.room-details {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex-wrap: wrap;
}

.room-details h3 {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  letter-spacing: 0.05em;
}

.mode-badge {
  padding: 0.2rem 0.5rem;
  font-size: 0.6rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  background: var(--accent);
  color: white;
}

.connection-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
.connection-dot.connected { background: #4CAF50; }
.connection-dot.disconnected { background: #9E9E9E; }
.connection-dot.error { background: #f44336; }

.member-count {
  font-size: 0.7rem;
  opacity: 0.5;
}

.room-actions {
  display: flex;
  gap: 0.5rem;
}

.action-btn {
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid rgba(0,0,0,0.1);
  cursor: pointer;
  transition: all 0.15s;
}

.action-btn:hover {
  background: rgba(0,0,0,0.05);
}

.action-btn.active {
  background: var(--accent);
  border-color: var(--accent);
  color: white;
}

.action-btn.muted {
  background: rgba(244, 67, 54, 0.1);
  border-color: rgba(244, 67, 54, 0.3);
  color: #d32f2f;
}

.action-btn svg {
  width: 18px;
  height: 18px;
}

/* Broadcast Toggle Button */
.broadcast-toggle {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  background: transparent;
  border: 1px solid rgba(0,0,0,0.15);
  font-family: inherit;
  font-size: 0.8rem;
  cursor: pointer;
  transition: all 0.15s;
}

.broadcast-toggle:hover {
  background: rgba(0,0,0,0.05);
}

.broadcast-toggle.active {
  background: #f44336;
  border-color: #f44336;
  color: white;
}

.broadcast-toggle svg {
  width: 18px;
  height: 18px;
}

/* Stats Bar */
.stats-bar {
  display: flex;
  gap: 0;
  padding: 0;
  background: rgba(0,0,0,0.03);
  border-bottom: 1px solid rgba(0,0,0,0.08);
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
  scrollbar-width: none;
}

.stats-bar::-webkit-scrollbar {
  display: none;
}

.stat {
  display: flex;
  flex-direction: column;
  padding: 0.5rem 1rem;
  border-right: 1px solid rgba(0,0,0,0.08);
  min-width: 80px;
}

.stat:last-child {
  border-right: none;
}

.stat-label {
  font-size: 0.6rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  opacity: 0.5;
}

.stat-value {
  font-size: 0.8rem;
  font-weight: 500;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

.stat-value.live {
  color: #f44336;
  font-weight: 600;
}

.stat-value.viewer {
  color: #2196F3;
}

.stat.status {
  background: rgba(0,0,0,0.02);
}

/* Video Layout */
.video-layout {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.video-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 1rem;
  padding: 1rem;
  overflow-y: auto;
  align-content: start;
}

.video-grid.single {
  grid-template-columns: 1fr;
  margin: 0 auto;
}

.video-cell {
  position: relative;
  aspect-ratio: 4/3;
  background: #1a1a1a;
  border-radius: 4px;
  overflow: hidden;
}

.video-cell.local {
  border: 2px solid var(--accent);
}

/* Viewer mode prompt */
.viewer-prompt {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  padding: 2rem;
  background: rgba(0,0,0,0.03);
  border: 2px dashed rgba(0,0,0,0.15);
  text-align: center;
  min-height: 200px;
}

.viewer-prompt svg {
  width: 48px;
  height: 48px;
  opacity: 0.3;
}

.viewer-prompt h4 {
  margin: 0;
  font-size: 1rem;
  font-weight: 500;
  letter-spacing: 0.05em;
}

.viewer-prompt p {
  margin: 0;
  font-size: 0.85rem;
  opacity: 0.6;
}

.start-broadcast-btn {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.6rem 1.2rem;
  background: var(--accent);
  color: white;
  border: none;
  font-family: inherit;
  font-size: 0.85rem;
  cursor: pointer;
  transition: all 0.15s;
  margin-top: 0.5rem;
}

.start-broadcast-btn:hover {
  background: var(--ink);
}

.start-broadcast-btn svg {
  width: 18px;
  height: 18px;
  opacity: 1;
}

.video-element,
.video-canvas {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.video-cell.local .video-element {
  transform: scaleX(-1);
}

.video-label {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 0.5rem 0.75rem;
  background: linear-gradient(transparent, rgba(0,0,0,0.7));
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.video-label .name {
  color: white;
  font-size: 0.8rem;
  font-weight: 500;
}

.broadcast-indicator {
  padding: 0.15rem 0.4rem;
  background: #f44336;
  color: white;
  font-size: 0.6rem;
  font-weight: 600;
  letter-spacing: 0.05em;
  border-radius: 2px;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.6; }
}

.muted-indicator {
  width: 16px;
  height: 16px;
  color: #f44336;
}

/* Participants Sidebar */
.participants-sidebar {
  display: none;
  flex-direction: column;
  background: rgba(255,255,255,0.3);
  border-left: 1px solid rgba(0,0,0,0.08);
}

.sidebar-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  font-size: 0.7rem;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  opacity: 0.5;
  border-bottom: 1px solid rgba(0,0,0,0.08);
}

.sidebar-header svg {
  width: 14px;
  height: 14px;
}

.participants-list {
  flex: 1;
  overflow-y: auto;
  padding: 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.participant {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.4rem;
  border-radius: 4px;
  transition: background 0.15s;
}

.participant:hover {
  background: rgba(0,0,0,0.03);
}

.participant.self {
  background: rgba(255, 95, 31, 0.05);
}

.participant-avatar {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 0.6rem;
  font-weight: 600;
  flex-shrink: 0;
}

.participant-name {
  flex: 1;
  font-size: 0.8rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.you-tag {
  font-size: 0.6rem;
  opacity: 0.4;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.online-indicator {
  width: 8px;
  height: 8px;
  background: #4CAF50;
  border-radius: 50%;
  flex-shrink: 0;
}

.live-dot {
  width: 8px;
  height: 8px;
  background: #f44336;
  border-radius: 50%;
  flex-shrink: 0;
  animation: pulse 2s infinite;
}

/* Error Banner */
.error-banner {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.75rem;
  background: rgba(244, 67, 54, 0.1);
  color: #d32f2f;
  font-size: 0.85rem;
  border-top: 1px solid rgba(244, 67, 54, 0.2);
}

.error-banner svg {
  width: 16px;
  height: 16px;
}

/* Responsive - Tablet and up */
@media (min-width: 768px) {
  .join-card {
    padding: 2.5rem;
  }

  .field input {
    font-size: 0.95rem;
  }

  .join-buttons {
    grid-template-columns: 1fr 1fr;
  }

  .video-layout {
    display: grid;
    grid-template-columns: 1fr 200px;
    flex-direction: unset;
  }

  .video-grid {
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  }

  .video-grid.single {
    max-width: 640px;
  }

  .participants-sidebar {
    display: flex;
  }

  .room-details {
    flex-wrap: nowrap;
  }
}
</style>
