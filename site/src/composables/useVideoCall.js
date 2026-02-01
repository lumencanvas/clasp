/**
 * useVideoCall - P2P WebRTC video calling composable
 * Uses CLASP for signaling, WebRTC for media transport
 */

import { ref, reactive, computed, onUnmounted } from 'vue'
import { useClasp } from './useClasp'

// ICE server configuration
const ICE_SERVERS = [
  { urls: 'stun:stun.l.google.com:19302' },
  { urls: 'stun:stun1.l.google.com:19302' },
]

/**
 * Create a video call composable instance
 */
export function useVideoCall() {
  const { connected, sessionId, subscribe, set, emit } = useClasp()

  // State
  const localStream = ref(null)
  const room = ref('')
  const inRoom = ref(false)
  const peers = reactive(new Map()) // peerId -> { connection, stream, name, audioEnabled, videoEnabled }
  const audioEnabled = ref(true)
  const videoEnabled = ref(true)
  const error = ref(null)
  const participants = ref(new Map()) // peerId -> { name, joinedAt }

  // Store current user name for presence updates
  let currentUserName = ''

  // Subscriptions
  let unsubPresence = null
  let unsubSignal = null
  let presenceInterval = null

  // ICE candidate queues per peer (queued before remote description is set)
  const iceCandidateQueues = new Map() // peerId -> RTCIceCandidate[]

  // Computed
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

  /**
   * Get user media (camera + mic)
   */
  async function getUserMedia(constraints = {}) {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        video: constraints.video ?? {
          width: { ideal: 640 },
          height: { ideal: 480 },
          frameRate: { ideal: 30 },
        },
        audio: constraints.audio ?? true,
      })
      localStream.value = stream
      return stream
    } catch (e) {
      error.value = `Camera access failed: ${e.message}`
      throw e
    }
  }

  /**
   * Stop local media tracks
   */
  function stopUserMedia() {
    if (localStream.value) {
      localStream.value.getTracks().forEach(track => track.stop())
      localStream.value = null
    }
  }

  /**
   * Join a video room
   */
  async function joinRoom(roomName, userName) {
    if (!connected.value || !sessionId.value) {
      error.value = 'Not connected to CLASP server'
      return false
    }

    if (!localStream.value) {
      error.value = 'No local media stream'
      return false
    }

    room.value = roomName
    currentUserName = userName || sessionId.value.slice(0, 8)

    // Subscribe to presence announcements
    const presencePattern = `/video/room/${roomName}/presence/*`
    unsubPresence = subscribe(presencePattern, (data, address) => {
      const peerId = address.split('/').pop()
      if (peerId === sessionId.value) return

      if (data === null) {
        // Peer left
        handlePeerLeft(peerId)
      } else {
        // Peer joined or updated
        handlePeerJoined(peerId, data)
      }
    })

    // Subscribe to signaling messages for us
    const signalPattern = `/video/room/${roomName}/signal/${sessionId.value}`
    unsubSignal = subscribe(signalPattern, (data, address) => {
      if (data && data.from && data.from !== sessionId.value) {
        handleSignal(data)
      }
    })

    // Announce our presence
    announcePresence()
    presenceInterval = setInterval(() => {
      announcePresence()
      pruneStaleParticipants()
    }, 10000)

    inRoom.value = true
    return true
  }

  /**
   * Leave the current room
   */
  function leaveRoom() {
    // Clear presence
    if (connected.value && sessionId.value && room.value) {
      set(`/video/room/${room.value}/presence/${sessionId.value}`, null)
    }

    // Close all peer connections
    for (const [peerId] of peers) {
      closePeerConnection(peerId)
    }
    peers.clear()
    participants.value.clear()
    iceCandidateQueues.clear()

    // Cleanup subscriptions
    if (unsubPresence) {
      unsubPresence()
      unsubPresence = null
    }
    if (unsubSignal) {
      unsubSignal()
      unsubSignal = null
    }
    if (presenceInterval) {
      clearInterval(presenceInterval)
      presenceInterval = null
    }

    room.value = ''
    inRoom.value = false
    currentUserName = ''
  }

  /**
   * Announce presence in the room
   */
  function announcePresence() {
    if (!connected.value || !sessionId.value || !room.value) return

    set(`/video/room/${room.value}/presence/${sessionId.value}`, {
      name: currentUserName || sessionId.value.slice(0, 8),
      joinedAt: Date.now(),
      lastSeen: Date.now(),
      audioEnabled: audioEnabled.value,
      videoEnabled: videoEnabled.value,
    })
  }

  /**
   * Handle a peer joining
   */
  function handlePeerJoined(peerId, data) {
    participants.value.set(peerId, { ...data, lastSeen: data.lastSeen || Date.now() })

    // If we don't have a connection to this peer, initiate one
    // Use lexicographic comparison for consistent initiator selection
    // The peer with the "larger" session ID initiates
    if (!peers.has(peerId) && sessionId.value.localeCompare(peerId) > 0) {
      createPeerConnection(peerId, data.name, true)
    }
  }

  /**
   * Handle a peer leaving
   */
  function handlePeerLeft(peerId) {
    participants.value.delete(peerId)
    closePeerConnection(peerId)
    peers.delete(peerId)
    iceCandidateQueues.delete(peerId)
  }

  /**
   * Prune participants whose lastSeen is older than 25 seconds
   */
  function pruneStaleParticipants() {
    const now = Date.now()
    const staleThreshold = 25_000
    const stale = []
    for (const [peerId, data] of participants.value) {
      if (now - (data.lastSeen || data.joinedAt || 0) > staleThreshold) {
        stale.push(peerId)
      }
    }
    stale.forEach(id => handlePeerLeft(id))
  }

  /**
   * Create a peer connection
   */
  async function createPeerConnection(peerId, peerName, initiator = false) {
    if (peers.has(peerId)) {
      return peers.get(peerId).connection
    }

    const connection = new RTCPeerConnection({ iceServers: ICE_SERVERS })

    // Initialize peer data in the map first
    peers.set(peerId, {
      connection,
      stream: null,
      name: peerName || peerId.slice(0, 8),
      audioEnabled: true,
      videoEnabled: true,
    })

    // Add local tracks
    if (localStream.value) {
      localStream.value.getTracks().forEach(track => {
        connection.addTrack(track, localStream.value)
      })
    }

    // Handle incoming tracks
    connection.ontrack = (event) => {
      const [remoteStream] = event.streams
      // Update the existing peer entry in the map
      const existingPeer = peers.get(peerId)
      if (existingPeer) {
        existingPeer.stream = remoteStream
        // Force reactivity update by setting a new object
        peers.set(peerId, { ...existingPeer })
      }
    }

    // Handle ICE candidates
    connection.onicecandidate = (event) => {
      if (event.candidate) {
        // Convert RTCIceCandidate to plain object for serialization
        const c = event.candidate
        sendSignal(peerId, {
          type: 'ice-candidate',
          candidate: {
            candidate: c.candidate,
            sdpMid: c.sdpMid,
            sdpMLineIndex: c.sdpMLineIndex,
          },
        })
      }
    }

    // Handle ICE connection state for better reconnection handling
    connection.oniceconnectionstatechange = () => {
      if (connection.iceConnectionState === 'failed') {
        // Attempt ICE restart
        if (initiator && connection.connectionState !== 'closed') {
          connection.restartIce()
        }
      }
    }

    // Handle connection state changes
    connection.onconnectionstatechange = () => {
      const state = connection.connectionState
      if (state === 'failed') {
        // Connection failed, clean up
        closePeerConnection(peerId)
        peers.delete(peerId)
      } else if (state === 'disconnected') {
        // Temporary disconnection, wait before cleaning up
        setTimeout(() => {
          if (connection.connectionState === 'disconnected') {
            closePeerConnection(peerId)
            peers.delete(peerId)
          }
        }, 5000)
      }
    }

    // If we're the initiator, create and send offer
    if (initiator) {
      try {
        const offer = await connection.createOffer()
        await connection.setLocalDescription(offer)
        // Convert RTCSessionDescription to plain object for serialization
        const desc = connection.localDescription
        sendSignal(peerId, {
          type: 'offer',
          sdp: { type: desc.type, sdp: desc.sdp },
        })
      } catch (e) {
        error.value = `Failed to create offer: ${e.message}`
      }
    }

    return connection
  }

  /**
   * Close a peer connection
   */
  function closePeerConnection(peerId) {
    const peer = peers.get(peerId)
    if (peer) {
      // Don't stop remote stream tracks - we don't own them
      // Just close the connection
      if (peer.connection && peer.connection.connectionState !== 'closed') {
        peer.connection.close()
      }
    }
  }

  /**
   * Send a signaling message to a peer
   */
  function sendSignal(peerId, data) {
    if (!connected.value || !sessionId.value || !room.value) return

    emit(`/video/room/${room.value}/signal/${peerId}`, {
      from: sessionId.value,
      ...data,
    })
  }

  /**
   * Drain queued ICE candidates for a peer after remote description is set
   */
  async function drainIceCandidateQueue(peerId) {
    const queue = iceCandidateQueues.get(peerId)
    if (!queue || queue.length === 0) return

    const peer = peers.get(peerId)
    if (!peer?.connection) return

    for (const candidate of queue) {
      try {
        await peer.connection.addIceCandidate(candidate)
      } catch (e) {
        console.warn('ICE candidate error (queued):', e)
      }
    }
    iceCandidateQueues.delete(peerId)
  }

  /**
   * Handle incoming signaling message
   */
  async function handleSignal(data) {
    const { from, type } = data

    if (type === 'offer') {
      // Received an offer, create connection and send answer
      const connection = await createPeerConnection(from, participants.value.get(from)?.name, false)

      try {
        await connection.setRemoteDescription(new RTCSessionDescription(data.sdp))
        await drainIceCandidateQueue(from)
        const answer = await connection.createAnswer()
        await connection.setLocalDescription(answer)
        const desc = connection.localDescription
        sendSignal(from, {
          type: 'answer',
          sdp: { type: desc.type, sdp: desc.sdp },
        })
      } catch (e) {
        error.value = `Failed to handle offer: ${e.message}`
      }
    } else if (type === 'answer') {
      // Received an answer to our offer
      const peer = peers.get(from)
      if (peer && peer.connection) {
        try {
          await peer.connection.setRemoteDescription(new RTCSessionDescription(data.sdp))
          await drainIceCandidateQueue(from)
        } catch (e) {
          error.value = `Failed to handle answer: ${e.message}`
        }
      }
    } else if (type === 'ice-candidate') {
      // Received ICE candidate - queue if remote description not yet set
      const peer = peers.get(from)
      if (peer && peer.connection) {
        if (peer.connection.remoteDescription) {
          try {
            await peer.connection.addIceCandidate(new RTCIceCandidate(data.candidate))
          } catch (e) {
            console.warn('ICE candidate error:', e)
          }
        } else {
          // Queue the candidate
          if (!iceCandidateQueues.has(from)) {
            iceCandidateQueues.set(from, [])
          }
          iceCandidateQueues.get(from).push(new RTCIceCandidate(data.candidate))
        }
      } else {
        // Peer connection not yet created, queue anyway
        if (!iceCandidateQueues.has(from)) {
          iceCandidateQueues.set(from, [])
        }
        iceCandidateQueues.get(from).push(new RTCIceCandidate(data.candidate))
      }
    }
  }

  /**
   * Toggle local audio
   */
  function toggleAudio() {
    if (localStream.value) {
      const audioTrack = localStream.value.getAudioTracks()[0]
      if (audioTrack) {
        audioTrack.enabled = !audioTrack.enabled
        audioEnabled.value = audioTrack.enabled
        announcePresence()
      }
    }
  }

  /**
   * Toggle local video
   */
  function toggleVideo() {
    if (localStream.value) {
      const videoTrack = localStream.value.getVideoTracks()[0]
      if (videoTrack) {
        videoTrack.enabled = !videoTrack.enabled
        videoEnabled.value = videoTrack.enabled
        announcePresence()
      }
    }
  }

  /**
   * Replace video track (e.g., switch cameras or share screen)
   */
  async function replaceVideoTrack(newTrack) {
    if (!localStream.value) return

    const oldTrack = localStream.value.getVideoTracks()[0]
    if (oldTrack) {
      localStream.value.removeTrack(oldTrack)
      oldTrack.stop()
    }

    localStream.value.addTrack(newTrack)

    // Update all peer connections
    for (const [, peer] of peers) {
      const sender = peer.connection?.getSenders().find(s => s.track?.kind === 'video')
      if (sender) {
        await sender.replaceTrack(newTrack)
      }
    }
  }

  /**
   * Share screen
   */
  async function shareScreen() {
    try {
      const screenStream = await navigator.mediaDevices.getDisplayMedia({
        video: true,
        audio: false,
      })
      const screenTrack = screenStream.getVideoTracks()[0]

      // When screen sharing stops, revert to camera
      screenTrack.onended = async () => {
        try {
          const cameraStream = await navigator.mediaDevices.getUserMedia({ video: true })
          await replaceVideoTrack(cameraStream.getVideoTracks()[0])
        } catch (e) {
          error.value = `Failed to restore camera: ${e.message}`
          console.error('Failed to restore camera after screen share:', e)
        }
      }

      await replaceVideoTrack(screenTrack)
    } catch (e) {
      error.value = `Screen share failed: ${e.message}`
    }
  }

  // Cleanup on unmount
  onUnmounted(() => {
    leaveRoom()
    stopUserMedia()
  })

  return {
    // State
    localStream,
    room,
    inRoom,
    peers,
    peerList,
    participantList,
    audioEnabled,
    videoEnabled,
    error,

    // Methods
    getUserMedia,
    stopUserMedia,
    joinRoom,
    leaveRoom,
    toggleAudio,
    toggleVideo,
    shareScreen,
  }
}
