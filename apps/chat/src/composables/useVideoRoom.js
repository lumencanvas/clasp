import { ref, reactive, computed, watch, onUnmounted } from 'vue'
import { useClasp } from './useClasp.js'
import { useIdentity } from './useIdentity.js'
import { ADDR, TTL } from '../lib/constants.js'

const ICE_SERVERS = [
  { urls: 'stun:stun.l.google.com:19302' },
  { urls: 'stun:stun1.l.google.com:19302' },
]

/**
 * Per-room video channel composable (P2P WebRTC)
 */
export function useVideoRoom(roomId) {
  const { connected, sessionId, subscribe, set, emit: claspEmit } = useClasp()
  const { displayName, avatarColor } = useIdentity()

  const localStream = ref(null)
  const inVideo = ref(false)
  const audioEnabled = ref(true)
  const videoEnabled = ref(true)
  const error = ref(null)
  const peers = reactive(new Map())
  const participants = ref(new Map())

  const iceCandidateQueues = new Map()

  let unsubPresence = null
  let unsubSignal = null
  let presenceInterval = null

  const peerList = computed(() => {
    return Array.from(peers.entries()).map(([id, peer]) => ({
      id,
      name: peer.name || id.slice(0, 8),
      stream: peer.stream,
      audioEnabled: peer.audioEnabled,
      videoEnabled: peer.videoEnabled,
      connectionState: peer.connection?.connectionState || 'new',
    }))
  })

  const participantList = computed(() => {
    return Array.from(participants.value.entries())
      .map(([id, data]) => ({ id, ...data }))
      .filter(p => p.id !== sessionId.value)
  })

  async function getUserMedia(constraints = {}) {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        video: constraints.video ?? { width: { ideal: 640 }, height: { ideal: 480 }, frameRate: { ideal: 30 } },
        audio: constraints.audio ?? true,
      })
      localStream.value = stream
      return stream
    } catch (e) {
      error.value = `Camera access failed: ${e.message}`
      throw e
    }
  }

  function stopUserMedia() {
    if (localStream.value) {
      localStream.value.getTracks().forEach(t => t.stop())
      localStream.value = null
    }
  }

  async function joinVideo() {
    if (!connected.value || !sessionId.value || !roomId.value) {
      error.value = 'Not connected'
      return false
    }
    if (!localStream.value) {
      error.value = 'No local media stream'
      return false
    }

    const rid = roomId.value

    // Subscribe to presence
    unsubPresence = subscribe(`${ADDR.ROOM}/${rid}/video/presence/*`, (data, address) => {
      const peerId = address.split('/').pop()
      if (peerId === sessionId.value) return

      if (data === null) {
        handlePeerLeft(peerId)
      } else {
        handlePeerJoined(peerId, data)
      }
    })

    // Subscribe to signaling messages for us
    unsubSignal = subscribe(`${ADDR.ROOM}/${rid}/video/signal/${sessionId.value}`, (data) => {
      if (data && data.from && data.from !== sessionId.value) {
        handleSignal(data)
      }
    })

    // Announce presence
    announcePresence()
    presenceInterval = setInterval(() => {
      announcePresence()
      pruneStaleParticipants()
    }, TTL.PRESENCE_HEARTBEAT)

    inVideo.value = true
    return true
  }

  function leaveVideo() {
    const rid = roomId.value

    if (connected.value && sessionId.value && rid) {
      set(`${ADDR.ROOM}/${rid}/video/presence/${sessionId.value}`, null)
    }

    for (const [peerId] of peers) {
      closePeerConnection(peerId)
    }
    peers.clear()
    participants.value.clear()
    iceCandidateQueues.clear()

    if (unsubPresence) { unsubPresence(); unsubPresence = null }
    if (unsubSignal) { unsubSignal(); unsubSignal = null }
    if (presenceInterval) { clearInterval(presenceInterval); presenceInterval = null }

    inVideo.value = false
  }

  function announcePresence() {
    if (!connected.value || !sessionId.value || !roomId.value) return
    set(`${ADDR.ROOM}/${roomId.value}/video/presence/${sessionId.value}`, {
      name: displayName.value,
      avatarColor: avatarColor.value,
      audioEnabled: audioEnabled.value,
      videoEnabled: videoEnabled.value,
      joinedAt: Date.now(),
      lastSeen: Date.now(),
    })
  }

  function handlePeerJoined(peerId, data) {
    participants.value.set(peerId, { ...data, lastSeen: data.lastSeen || Date.now() })
    participants.value = new Map(participants.value)

    if (!peers.has(peerId) && sessionId.value.localeCompare(peerId) > 0) {
      createPeerConnection(peerId, data.name, true)
    }
  }

  function handlePeerLeft(peerId) {
    participants.value.delete(peerId)
    participants.value = new Map(participants.value)
    closePeerConnection(peerId)
    peers.delete(peerId)
    iceCandidateQueues.delete(peerId)
  }

  function pruneStaleParticipants() {
    const now = Date.now()
    const stale = []
    for (const [peerId, data] of participants.value) {
      if (now - (data.lastSeen || data.joinedAt || 0) > TTL.PRESENCE_STALE) {
        stale.push(peerId)
      }
    }
    stale.forEach(id => handlePeerLeft(id))
  }

  async function createPeerConnection(peerId, peerName, initiator = false) {
    if (peers.has(peerId)) return peers.get(peerId).connection

    const connection = new RTCPeerConnection({ iceServers: ICE_SERVERS })

    peers.set(peerId, {
      connection,
      stream: null,
      name: peerName || peerId.slice(0, 8),
      audioEnabled: true,
      videoEnabled: true,
    })

    if (localStream.value) {
      localStream.value.getTracks().forEach(track => {
        connection.addTrack(track, localStream.value)
      })
    }

    connection.ontrack = (event) => {
      const [remoteStream] = event.streams
      const existing = peers.get(peerId)
      if (existing) {
        existing.stream = remoteStream
        peers.set(peerId, { ...existing })
      }
    }

    connection.onicecandidate = (event) => {
      if (event.candidate) {
        const c = event.candidate
        sendSignal(peerId, {
          type: 'ice-candidate',
          candidate: { candidate: c.candidate, sdpMid: c.sdpMid, sdpMLineIndex: c.sdpMLineIndex },
        })
      }
    }

    connection.oniceconnectionstatechange = () => {
      if (connection.iceConnectionState === 'failed' && initiator && connection.connectionState !== 'closed') {
        connection.restartIce()
      }
    }

    connection.onconnectionstatechange = () => {
      const state = connection.connectionState
      if (state === 'failed') {
        closePeerConnection(peerId)
        peers.delete(peerId)
      } else if (state === 'disconnected') {
        setTimeout(() => {
          if (connection.connectionState === 'disconnected') {
            closePeerConnection(peerId)
            peers.delete(peerId)
          }
        }, TTL.DISCONNECT_GRACE)
      }
    }

    if (initiator) {
      try {
        const offer = await connection.createOffer()
        await connection.setLocalDescription(offer)
        const desc = connection.localDescription
        sendSignal(peerId, { type: 'offer', sdp: { type: desc.type, sdp: desc.sdp } })
      } catch (e) {
        error.value = `Failed to create offer: ${e.message}`
      }
    }

    return connection
  }

  function closePeerConnection(peerId) {
    const peer = peers.get(peerId)
    if (peer?.connection && peer.connection.connectionState !== 'closed') {
      peer.connection.close()
    }
  }

  function sendSignal(peerId, data) {
    if (!connected.value || !sessionId.value || !roomId.value) return
    claspEmit(`${ADDR.ROOM}/${roomId.value}/video/signal/${peerId}`, {
      from: sessionId.value,
      ...data,
    })
  }

  async function drainIceCandidateQueue(peerId) {
    const queue = iceCandidateQueues.get(peerId)
    if (!queue || queue.length === 0) return
    const peer = peers.get(peerId)
    if (!peer?.connection) return

    for (const candidate of queue) {
      try { await peer.connection.addIceCandidate(candidate) } catch (e) { /* ignore */ }
    }
    iceCandidateQueues.delete(peerId)
  }

  async function handleSignal(data) {
    const { from, type } = data

    if (type === 'offer') {
      const connection = await createPeerConnection(from, participants.value.get(from)?.name, false)
      try {
        await connection.setRemoteDescription(new RTCSessionDescription(data.sdp))
        await drainIceCandidateQueue(from)
        const answer = await connection.createAnswer()
        await connection.setLocalDescription(answer)
        const desc = connection.localDescription
        sendSignal(from, { type: 'answer', sdp: { type: desc.type, sdp: desc.sdp } })
      } catch (e) {
        error.value = `Failed to handle offer: ${e.message}`
      }
    } else if (type === 'answer') {
      const peer = peers.get(from)
      if (peer?.connection) {
        try {
          await peer.connection.setRemoteDescription(new RTCSessionDescription(data.sdp))
          await drainIceCandidateQueue(from)
        } catch (e) {
          error.value = `Failed to handle answer: ${e.message}`
        }
      }
    } else if (type === 'ice-candidate') {
      const peer = peers.get(from)
      if (peer?.connection?.remoteDescription) {
        try { await peer.connection.addIceCandidate(new RTCIceCandidate(data.candidate)) } catch (e) { /* ignore */ }
      } else {
        if (!iceCandidateQueues.has(from)) iceCandidateQueues.set(from, [])
        iceCandidateQueues.get(from).push(new RTCIceCandidate(data.candidate))
      }
    }
  }

  function toggleAudio() {
    if (!localStream.value) return
    const track = localStream.value.getAudioTracks()[0]
    if (track) {
      track.enabled = !track.enabled
      audioEnabled.value = track.enabled
      announcePresence()
    }
  }

  function toggleVideo() {
    if (!localStream.value) return
    const track = localStream.value.getVideoTracks()[0]
    if (track) {
      track.enabled = !track.enabled
      videoEnabled.value = track.enabled
      announcePresence()
    }
  }

  async function replaceVideoTrack(newTrack) {
    if (!localStream.value) return
    const oldTrack = localStream.value.getVideoTracks()[0]
    if (oldTrack) {
      localStream.value.removeTrack(oldTrack)
      oldTrack.stop()
    }
    localStream.value.addTrack(newTrack)

    for (const [, peer] of peers) {
      const sender = peer.connection?.getSenders().find(s => s.track?.kind === 'video')
      if (sender) await sender.replaceTrack(newTrack)
    }
  }

  async function shareScreen() {
    try {
      const screenStream = await navigator.mediaDevices.getDisplayMedia({ video: true, audio: false })
      const screenTrack = screenStream.getVideoTracks()[0]

      screenTrack.onended = async () => {
        try {
          const cameraStream = await navigator.mediaDevices.getUserMedia({ video: true })
          await replaceVideoTrack(cameraStream.getVideoTracks()[0])
        } catch (e) {
          error.value = `Failed to restore camera: ${e.message}`
        }
      }

      await replaceVideoTrack(screenTrack)
    } catch (e) {
      error.value = `Screen share failed: ${e.message}`
    }
  }

  // Cleanup on room change
  watch(roomId, (newId, oldId) => {
    if (oldId && inVideo.value) {
      leaveVideo()
      stopUserMedia()
    }
  })

  onUnmounted(() => {
    if (inVideo.value) leaveVideo()
    stopUserMedia()
  })

  return {
    localStream,
    inVideo,
    audioEnabled,
    videoEnabled,
    error,
    peers,
    peerList,
    participantList,
    getUserMedia,
    stopUserMedia,
    joinVideo,
    leaveVideo,
    toggleAudio,
    toggleVideo,
    shareScreen,
  }
}
