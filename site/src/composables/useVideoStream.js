/**
 * useVideoStream - CLASP Relay mode video streaming composable
 * Uses WebCodecs for encoding/decoding and CLASP for transport
 */

import { ref, reactive, computed, onUnmounted } from 'vue'
import { useClasp } from './useClasp'
import {
  chunkFrame,
  encodeChunkForTransport,
  decodeChunkFromTransport,
  ChunkAssembler,
  createSequenceGenerator,
} from '../lib/videoChunker'

// Quality presets
export const QUALITY_PRESETS = {
  low: {
    width: 640,
    height: 480,
    bitrate: 400_000,
    framerate: 24,
  },
  medium: {
    width: 1280,
    height: 720,
    bitrate: 1_200_000,
    framerate: 30,
  },
  high: {
    width: 1920,
    height: 1080,
    bitrate: 3_000_000,
    framerate: 30,
  },
}

/**
 * Create a video stream composable instance
 */
export function useVideoStream() {
  const { connected, sessionId, subscribe, subscribeRaw, set, stream: claspStream, emit } = useClasp()

  // State
  const localStream = ref(null)
  const room = ref('')
  const inRoom = ref(false)
  const isBroadcasting = ref(false)
  const error = ref(null)
  const participants = ref(new Map()) // peerId -> { name, isBroadcaster }
  const remoteStreams = reactive(new Map()) // peerId -> { canvas, assembler, decoder }

  // Quality settings
  const quality = reactive({
    preset: 'medium',
    width: 1280,
    height: 720,
    bitrate: 1_200_000,
    framerate: 30,
  })

  // Stats
  const stats = reactive({
    framesSent: 0,
    framesReceived: 0,
    bytesPerSecond: 0,
    keyFrameRequests: 0,
  })

  // Internal state
  let encoder = null
  let encoderConfigured = false
  let codecDescription = null // SPS/PPS for H.264
  let videoTrack = null
  let trackProcessor = null
  let frameReader = null
  let unsubPresence = null
  let unsubKeyframeRequest = null
  let unsubStreams = new Map()
  let presenceInterval = null
  let lastKeyFrameTime = 0
  let bytesSentInSecond = 0
  let statsInterval = null
  let currentUserName = ''
  let seqGenerator = null
  let canvasCaptureCleanup = null

  // Computed
  const participantList = computed(() => {
    return Array.from(participants.value.entries())
      .map(([id, data]) => ({ id, ...data }))
      .filter(p => p.id !== sessionId.value)
  })

  const broadcasterList = computed(() => {
    return participantList.value.filter(p => p.isBroadcaster)
  })

  /**
   * Check WebCodecs support
   */
  function checkWebCodecsSupport() {
    if (typeof VideoEncoder === 'undefined' || typeof VideoDecoder === 'undefined') {
      error.value = 'WebCodecs not supported in this browser'
      return false
    }
    return true
  }

  /**
   * Get user media (camera + mic)
   */
  async function getUserMedia(constraints = {}) {
    try {
      const mediaConstraints = {
        video: constraints.video ?? {
          width: { ideal: quality.width },
          height: { ideal: quality.height },
          frameRate: { ideal: quality.framerate },
        },
        audio: constraints.audio ?? true,
      }

      const stream = await navigator.mediaDevices.getUserMedia(mediaConstraints)
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
   * Apply quality preset
   */
  function setQuality(preset) {
    if (QUALITY_PRESETS[preset]) {
      const settings = QUALITY_PRESETS[preset]
      quality.preset = preset
      quality.width = settings.width
      quality.height = settings.height
      quality.bitrate = settings.bitrate
      quality.framerate = settings.framerate

      // Reconfigure encoder if broadcasting
      if (isBroadcasting.value && encoder) {
        configureEncoder()
      }
    }
  }

  /**
   * Set custom quality settings
   */
  function setCustomQuality(settings) {
    quality.preset = 'custom'
    if (settings.width) quality.width = settings.width
    if (settings.height) quality.height = settings.height
    if (settings.bitrate) quality.bitrate = settings.bitrate
    if (settings.framerate) quality.framerate = settings.framerate

    if (isBroadcasting.value && encoder) {
      configureEncoder()
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

    room.value = roomName
    currentUserName = userName || sessionId.value.slice(0, 8)

    // Subscribe to presence
    const presencePattern = `/video/relay/${roomName}/presence/*`
    unsubPresence = subscribe(presencePattern, (data, address) => {
      const peerId = address.split('/').pop()
      if (peerId === sessionId.value) return

      if (data === null) {
        handlePeerLeft(peerId)
      } else {
        handlePeerJoined(peerId, data)
      }
    })

    // Announce presence (announce quickly at first, then slow down)
    announcePresence(false)
    // Announce again in 1 second to catch late subscribers
    setTimeout(() => announcePresence(isBroadcasting.value), 1000)
    // Then every 5 seconds, also prune stale peers
    presenceInterval = setInterval(() => {
      announcePresence(isBroadcasting.value)
      pruneStaleParticipants()
    }, 5000)

    // Start stats tracking
    statsInterval = setInterval(updateStats, 1000)

    inRoom.value = true
    return true
  }

  /**
   * Leave the current room
   */
  function leaveRoom() {
    stopBroadcasting()

    // Clear presence
    if (connected.value && sessionId.value && room.value) {
      set(`/video/relay/${room.value}/presence/${sessionId.value}`, null)
    }

    // Cleanup subscriptions
    for (const [, unsub] of unsubStreams) {
      unsub()
    }
    unsubStreams.clear()

    if (unsubPresence) {
      unsubPresence()
      unsubPresence = null
    }
    if (presenceInterval) {
      clearInterval(presenceInterval)
      presenceInterval = null
    }
    if (statsInterval) {
      clearInterval(statsInterval)
      statsInterval = null
    }

    // Cleanup remote streams
    for (const [peerId] of remoteStreams) {
      cleanupRemoteStream(peerId)
    }
    remoteStreams.clear()
    participants.value.clear()

    room.value = ''
    inRoom.value = false
    currentUserName = ''
  }

  /**
   * Announce presence in the room
   */
  function announcePresence(broadcasting) {
    if (!connected.value || !sessionId.value || !room.value) return

    set(`/video/relay/${room.value}/presence/${sessionId.value}`, {
      name: currentUserName || sessionId.value.slice(0, 8),
      isBroadcaster: broadcasting,
      joinedAt: Date.now(),
      lastSeen: Date.now(),
      quality: quality.preset,
    })
  }

  /**
   * Handle peer joining
   */
  function handlePeerJoined(peerId, data) {
    const existing = participants.value.get(peerId)
    const wasBroadcasting = existing?.isBroadcaster

    participants.value.set(peerId, { ...data, lastSeen: data.lastSeen || Date.now() })

    // If peer is broadcasting, subscribe to their stream
    if (data.isBroadcaster && !wasBroadcasting) {
      subscribeToStream(peerId)
    }
    // If peer stopped broadcasting, clean up their stream
    if (!data.isBroadcaster && wasBroadcasting) {
      cleanupRemoteStream(peerId)
      const unsub = unsubStreams.get(peerId)
      if (unsub) {
        unsub()
        unsubStreams.delete(peerId)
      }
    }
  }

  /**
   * Handle peer leaving
   */
  function handlePeerLeft(peerId) {
    participants.value.delete(peerId)
    cleanupRemoteStream(peerId)

    const unsub = unsubStreams.get(peerId)
    if (unsub) {
      unsub()
      unsubStreams.delete(peerId)
    }
  }

  /**
   * Prune participants whose lastSeen is older than 20 seconds
   */
  function pruneStaleParticipants() {
    const now = Date.now()
    const staleThreshold = 20_000
    const stale = []
    for (const [peerId, data] of participants.value) {
      if (now - (data.lastSeen || data.joinedAt || 0) > staleThreshold) {
        stale.push(peerId)
      }
    }
    stale.forEach(id => handlePeerLeft(id))
  }

  /**
   * Get the CLASP stream address for a peer
   */
  function getStreamAddress(peerId) {
    if (!room.value) return null
    return `/video/relay/${room.value}/stream/${peerId}`
  }

  /**
   * Subscribe to a peer's video stream
   */
  function subscribeToStream(peerId) {
    if (unsubStreams.has(peerId)) return

    // Create canvas for rendering
    const canvas = document.createElement('canvas')
    canvas.width = quality.width
    canvas.height = quality.height
    const ctx = canvas.getContext('2d')

    // Decoder state
    let decoderConfigured = false
    let waitingForKeyframe = false
    let remoteCodecDescription = null

    // Create decoder
    const decoder = new VideoDecoder({
      output: (frame) => {
        if (canvas.width !== frame.displayWidth || canvas.height !== frame.displayHeight) {
          canvas.width = frame.displayWidth
          canvas.height = frame.displayHeight
        }
        ctx.drawImage(frame, 0, 0)
        frame.close()
        stats.framesReceived++
      },
      error: (e) => {
        console.error('[VideoStream] Decoder error:', e)
        // Reset decoder and request keyframe
        decoderConfigured = false
        waitingForKeyframe = true
        try {
          decoder.reset()
        } catch (resetErr) {
          // ignore reset errors
        }
        requestKeyFrame(peerId)
      },
    })

    /**
     * Decode a complete assembled frame directly (no JitterBuffer)
     */
    function decodeFrame(frame) {
      try {
        // Store codec description if present
        if (frame.description) {
          remoteCodecDescription = frame.description
        }

        // Configure decoder on keyframe
        if (!decoderConfigured && frame.frameType === 'key') {
          const decoderConfig = {
            codec: 'avc1.42001e',
            optimizeForLatency: true,
            hardwareAcceleration: 'prefer-software',
          }
          if (remoteCodecDescription) {
            decoderConfig.description = remoteCodecDescription
          }
          decoder.configure(decoderConfig)
          decoderConfigured = true
          waitingForKeyframe = false
        }

        // Skip delta frames if waiting for keyframe
        if (waitingForKeyframe && frame.frameType !== 'key') {
          return
        }

        if (decoderConfigured) {
          decoder.decode(new EncodedVideoChunk({
            type: frame.frameType,
            timestamp: frame.timestamp,
            data: frame.data,
          }))
        }
      } catch (e) {
        console.error('[VideoStream] Decode error:', e)
        decoderConfigured = false
        waitingForKeyframe = true
        try {
          decoder.reset()
        } catch (resetErr) {
          // ignore
        }
        requestKeyFrame(peerId)
      }
    }

    // Create chunk assembler - frames go directly to decoder
    const assembler = new ChunkAssembler({
      onFrame: decodeFrame,
      onError: (e) => {
        console.error('[VideoStream] Assembly error:', e)
        requestKeyFrame(peerId)
      },
    })

    // Store remote stream state
    remoteStreams.set(peerId, {
      canvas,
      ctx,
      decoder,
      assembler,
    })

    // Subscribe to stream chunks using subscribeRaw (bypasses logging/reactive overhead)
    const streamPattern = `/video/relay/${room.value}/stream/${peerId}`

    const unsub = subscribeRaw(streamPattern, (data) => {
      try {
        if (data && data.data) {
          const chunk = decodeChunkFromTransport(data)
          assembler.addChunk(chunk)
        }
      } catch (e) {
        console.error('[VideoStream] Chunk processing error:', e)
      }
    })

    unsubStreams.set(peerId, unsub)
  }

  /**
   * Cleanup a remote stream
   */
  function cleanupRemoteStream(peerId) {
    const remote = remoteStreams.get(peerId)
    if (remote) {
      remote.assembler?.clear()
      if (remote.decoder?.state !== 'closed') {
        try {
          remote.decoder?.close()
        } catch (e) {
          // Ignore close errors
        }
      }
    }
    remoteStreams.delete(peerId)
  }

  /**
   * Request a key frame from a broadcaster
   */
  function requestKeyFrame(peerId) {
    if (!connected.value || !room.value) return
    stats.keyFrameRequests++
    emit(`/video/relay/${room.value}/request-keyframe/${peerId}`, {
      from: sessionId.value,
    })
  }

  /**
   * Start broadcasting video
   */
  async function startBroadcasting() {
    if (!checkWebCodecsSupport()) return false
    if (!localStream.value) {
      error.value = 'No local media stream'
      return false
    }

    // Create sequence generator for this broadcast session
    seqGenerator = createSequenceGenerator()

    // Create encoder
    encoder = new VideoEncoder({
      output: handleEncodedFrame,
      error: (e) => {
        error.value = `Encoder error: ${e.message}`
        console.error('Encoder error:', e)
      },
    })

    const configured = await configureEncoder()
    if (!configured) {
      return false
    }

    // Get video track and start processing
    videoTrack = localStream.value.getVideoTracks()[0]
    if (!videoTrack) {
      error.value = 'No video track available'
      return false
    }

    // Set broadcasting state BEFORE starting frame processing
    isBroadcasting.value = true
    announcePresence(true)

    // Listen for key frame requests
    const requestPattern = `/video/relay/${room.value}/request-keyframe/${sessionId.value}`
    unsubKeyframeRequest = subscribe(requestPattern, () => {
      // Force next frame to be a key frame
      lastKeyFrameTime = 0
    })

    // Use MediaStreamTrackProcessor for frame access
    if (typeof MediaStreamTrackProcessor !== 'undefined') {
      trackProcessor = new MediaStreamTrackProcessor({ track: videoTrack })
      frameReader = trackProcessor.readable.getReader()

      processFrames()
    } else {
      // Fallback: use canvas capture
      startCanvasCapture()
    }

    return true
  }

  /**
   * Process frames from MediaStreamTrackProcessor
   */
  async function processFrames() {
    while (isBroadcasting.value) {
      try {
        const { value: frame, done } = await frameReader.read()
        if (done || !isBroadcasting.value) {
          frame?.close()
          break
        }

        if (encoder && encoderConfigured && encoder.encodeQueueSize < 5) {
          // Force key frame periodically (every 3 seconds)
          const now = Date.now()
          const forceKeyFrame = now - lastKeyFrameTime > 3000
          if (forceKeyFrame) {
            lastKeyFrameTime = now
          }

          encoder.encode(frame, { keyFrame: forceKeyFrame })
        }
        frame.close()
      } catch (e) {
        if (isBroadcasting.value) {
          console.error('Frame processing error:', e)
        }
        break
      }
    }

    // Clean up reader
    if (frameReader) {
      try {
        frameReader.releaseLock()
      } catch (e) {
        // Ignore release errors
      }
      frameReader = null
    }
  }

  /**
   * Configure the video encoder
   */
  async function configureEncoder() {
    if (!encoder) return false

    const config = {
      codec: 'avc1.42001e', // H.264 Constrained Baseline (wider WebCodecs compatibility)
      width: quality.width,
      height: quality.height,
      bitrate: quality.bitrate,
      framerate: quality.framerate,
      latencyMode: 'realtime',
      hardwareAcceleration: 'prefer-hardware',
      avc: { format: 'annexb' }, // Include SPS/PPS in keyframes
    }

    try {
      const support = await VideoEncoder.isConfigSupported(config)
      if (!support.supported) {
        // Try with software encoding
        config.hardwareAcceleration = 'prefer-software'
        const softwareSupport = await VideoEncoder.isConfigSupported(config)
        if (!softwareSupport.supported) {
          error.value = 'H.264 encoding not supported'
          return false
        }
      }

      encoder.configure(config)
      encoderConfigured = true
      return true
    } catch (e) {
      error.value = `Encoder configuration failed: ${e.message}`
      encoderConfigured = false
      return false
    }
  }

  /**
   * Fallback: Canvas-based frame capture
   */
  function startCanvasCapture() {
    const video = document.createElement('video')
    video.srcObject = localStream.value
    video.muted = true

    const canvas = document.createElement('canvas')
    canvas.width = quality.width
    canvas.height = quality.height
    const ctx = canvas.getContext('2d')

    let captureInterval = null

    video.onloadedmetadata = () => {
      video.play().then(() => {
        captureInterval = setInterval(() => {
          if (!isBroadcasting.value) {
            clearInterval(captureInterval)
            return
          }

          ctx.drawImage(video, 0, 0, canvas.width, canvas.height)

          // Create VideoFrame from canvas
          try {
            const frame = new VideoFrame(canvas, {
              timestamp: performance.now() * 1000,
            })

            if (encoder && encoderConfigured && encoder.encodeQueueSize < 5) {
              const now = Date.now()
              const forceKeyFrame = now - lastKeyFrameTime > 3000
              if (forceKeyFrame) lastKeyFrameTime = now

              encoder.encode(frame, { keyFrame: forceKeyFrame })
            }
            frame.close()
          } catch (e) {
            console.error('Canvas capture error:', e)
          }
        }, 1000 / quality.framerate)
      })
    }

    // Store cleanup function
    canvasCaptureCleanup = () => {
      if (captureInterval) {
        clearInterval(captureInterval)
      }
      video.pause()
      video.srcObject = null
    }
  }

  /**
   * Handle encoded video frame from encoder
   */
  function handleEncodedFrame(chunk, metadata) {
    if (!connected.value || !room.value || !sessionId.value) return

    // Capture SPS/PPS codec description from keyframe metadata
    if (metadata?.decoderConfig?.description) {
      const desc = metadata.decoderConfig.description
      codecDescription = (desc instanceof ArrayBuffer)
        ? new Uint8Array(desc)
        : new Uint8Array(desc.buffer || desc)
    }

    // Copy chunk data
    const data = new Uint8Array(chunk.byteLength)
    chunk.copyTo(data)

    // Track stats
    bytesSentInSecond += data.byteLength
    stats.framesSent++

    // Chunk and send via CLASP
    const chunks = chunkFrame(data, chunk.type, chunk.timestamp, 16000, seqGenerator)
    const address = `/video/relay/${room.value}/stream/${sessionId.value}`

    // Attach codec description to first chunk of keyframes
    if (chunk.type === 'key' && codecDescription && chunks.length > 0) {
      chunks[0].description = codecDescription
    }

    chunks.forEach(c => {
      const encoded = encodeChunkForTransport(c)
      claspStream(address, encoded)
    })
  }

  /**
   * Stop broadcasting
   */
  function stopBroadcasting() {
    const wasBroadcasting = isBroadcasting.value
    isBroadcasting.value = false

    // Clean up frame reader
    if (frameReader) {
      try {
        frameReader.releaseLock()
      } catch (e) {
        // Ignore release errors
      }
      frameReader = null
    }

    // Clean up canvas capture
    if (canvasCaptureCleanup) {
      canvasCaptureCleanup()
      canvasCaptureCleanup = null
    }

    // Clean up encoder
    if (encoder) {
      if (encoder.state !== 'closed') {
        try {
          encoder.close()
        } catch (e) {
          // Ignore close errors
        }
      }
      encoder = null
      encoderConfigured = false
    }

    // Clean up keyframe request subscription
    if (unsubKeyframeRequest) {
      unsubKeyframeRequest()
      unsubKeyframeRequest = null
    }

    trackProcessor = null
    videoTrack = null
    seqGenerator = null
    codecDescription = null

    // Update presence if we were broadcasting
    if (wasBroadcasting && inRoom.value) {
      announcePresence(false)
    }
  }

  /**
   * Update stats
   */
  function updateStats() {
    stats.bytesPerSecond = bytesSentInSecond
    bytesSentInSecond = 0
  }

  /**
   * Get a remote stream's canvas element
   */
  function getRemoteCanvas(peerId) {
    return remoteStreams.get(peerId)?.canvas || null
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
    isBroadcasting,
    error,
    participants,
    participantList,
    broadcasterList,
    remoteStreams,
    quality,
    stats,

    // Quality
    QUALITY_PRESETS,
    setQuality,
    setCustomQuality,

    // Methods
    checkWebCodecsSupport,
    getUserMedia,
    stopUserMedia,
    joinRoom,
    leaveRoom,
    startBroadcasting,
    stopBroadcasting,
    getRemoteCanvas,
    getStreamAddress,
  }
}
